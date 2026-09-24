---
_type: Book
_label: "Using HCL in a Go Application"
url: "https://hcl.readthedocs.io/en/latest/go.html"
tags:
  - "dsl"
  - "structural matching"
  - "config language"
---

::: tip
Key architectural decision in HCL parser:

- **Structural decoding** and **expression evaluation** are separate phases.
  - You first decode the body into blocks and attributes (structural).
  - Then evaluate the attribute expressions (semantic).

This separation lets you control what variables and functions are available at each point.
:::

Check the pipeline in ${fref('./index.td')}.

- HCL is:
  - Go-only.
  - Primarily a library.
  - Not a standalone tool.
- Some use cases:
  - For simple cases, it decodes directly into Go structs, similar to `encoding/json`.
  - For complex cases, it exposes lower-level APIs: For example, plugin systems where the schema comes from outside the application (a plugin, user config, external registry) and is not known until runtime.

## Parsing

::: note
Input: raw bytes (HCL or JSON file/src)
Output: `*hcl.File` containing an opaque `hcl.Body`

- No schema required.
- No validity check.
- The parser does not know what is valid: it only recognizes syntax.
- JSON ambiguity (block vs object expression) is left unresolved until decoding.
- The `hcl.Body` cannot be inspected directly: it is a sealed type that only yields content when a schema is provided.
:::

- **`hclparse.NewParser() *Parser`**: creates a parser instance that holds a `map[string]*hcl.File` cache keyed by filename. A single instance is meant for one load pass: calling parse for the same file always returns the same cached pointer. To pick up file changes, create a new parser.
- **`ParseHCLFile(filename string) (*hcl.File, Diagnostics)`**: reads from disk and parses native HCL syntax.
  ```go
  parser := hclparse.NewParser()
  f, diags := parser.ParseHCLFile("server.conf")
  ```
  - `f` is an `*hcl.File`: an opaque abstract representation of the file, ready to be decoded.
  - `f.Body` is opaque: you cannot inspect it without providing a schema.
- **`ParseHCL(src []byte, filename string) (*hcl.File, Diagnostics)`**: same as `ParseHCLFile` but takes bytes directly instead of reading from disk.
- **`ParseJSONFile` / `ParseJSON`**: parses JSON syntax into the same `*hcl.File` representation.
  - JSON is ambiguous in places (e.g. whether a JSON object is a nested block or an object expression).
  - The schema resolves the ambiguity at decode time.

## Diagnostic Messages

- `hcl.Diagnostics` replaces Go's `error`.
  - It can carry both errors and warnings together..
  - While Go's `error` cannot express warnings without violating the nil-means-success convention.
- Each diagnostic carries:
  - Severity.
  - Summary.
  - Detail.
  - Source range.
  - Optional suggestions.
- Diagnostics accumulate across all processing phases:
  1. Parsing.
  2. Structural decoding.
  3. Expression evaluation
  all produce diagnostics that should be collected into one result.

### The Standard Accumulation Pattern

```go
func process() hcl.Diagnostics {
  var diags hcl.Diagnostics

  f, moreDiags := parser.ParseHCLFile("example.conf")
  diags = append(diags, moreDiags...)  // always append, warnings may be present
  if diags.HasErrors() {
    return diags  // early return if we cannot safely continue
  }

  // ...
  return diags
}
```

- A common variant: collect diagnostics in a loop over multiple files, using `continue` on errors but completing all iterations to surface all problems at once.

### Render Diagnostics to a Terminal

```go
wr := hcl.NewDiagnosticTextWriter(
  os.Stdout,
  parser.Files(),  // parser's file cache, used to include source snippets
  78,              // wrapping width
  true,            // colored output
)
wr.WriteDiagnostics(diags)
```

- `parser.Files()` is why you pass the parser around.
  - It holds the source cache needed to show relevant lines in error messages.
- Output includes:
  1. Source snippet.
  2. Line/column.
  3. Optional "did you mean" suggestions.

> Fowler's `Notification` pattern: collect all errors into a result object instead of stopping at the first.

## Decoding

::: note
Input: `hcl.Body` + schema
Output: typed blocks and attributes. Attribute values come back as `hcl.Expression` (unevaluated).

- This is where the schema is applied: block types, field names, and types are validated here.
- JSON ambiguity is resolved here: the schema tells the decoder whether a JSON object is a block or an object value.
- Expressions are not yet evaluated:
  - You get `hcl.Expression` handles, not concrete values.
  - Evaluation is a separate step.
- Three approaches depending on how much you know at compile time:
  - **gohcl** = Embedded Translation.

    Decode directly into your model.
  - **hcldec** = Tree Construction.

    Build an intermediate `cty.Value` tree.
  - **Low-level** = manual walk.

    Full control over each step.
:::

### 1. gohcl (struct tags)

- Best when the schema is known at compile time.
- Decodes into Go structs using reflection. The struct field tags imply the schema.

The struct definition:

```go
type Config struct {
  IOMode   string          `hcl:"io_mode"`
  Services []ServiceConfig `hcl:"service,block"`
}

type ServiceConfig struct {
  Type       string `hcl:"type,label"`
  Name       string `hcl:"name,label"`
  ListenAddr string `hcl:"listen_addr"`
}
```

Matches this HCL input:

```hcl
io_mode = "async"

service "http" "web_proxy" {
  listen_addr = "127.0.0.1:8080"
}
```

Decoded with:

```go
var c Config
moreDiags := gohcl.DecodeBody(f.Body, nil, &c)
diags = append(diags, moreDiags...)
```

Tag format: `hcl:"<hcl-name>,<kind>"`

- `<hcl-name>`: the name as it appears in the HCL input file (e.g. `io_mode`, `service`).
- `<kind>`: how this field maps to HCL structure:
  - `attr` (**default**, can be omitted) maps to an HCL attribute: `io_mode = "async"`.
  - `block` maps to an HCL block type: `service "http" "web_proxy" { ... }`.
    Field type must be a struct or slice of struct.
  - `label` maps to a block label.
    - Labels are the identifiers after the block type keyword (`"http"`, `"web_proxy"`).
    - The field name is only used in error messages when the wrong number of labels is given.
  - `remain` captures all unmatched body content into an `hcl.Body` for a later decode pass, potentially with a different eval context. Useful for two-pass decoding where different parts need different schemas or eval contexts:
    ```go
    // Phase 1: top-level only knows block type
    type ProviderBlock struct {
    Type   string   `hcl:"type,label"`
    Remain hcl.Body `hcl:",remain"`  // plugin-specific fields deferred
    }
    var c Config
    gohcl.DecodeBody(f.Body, nil, &c)

    // Phase 2: plugin decodes its own portion with its own schema
    var awsCfg AWSConfig
    gohcl.DecodeBody(c.Providers[0].Remain, awsEvalCtx, &awsCfg)
    ```
- All fields are required by default. Pointer type makes a field optional (`nil` = absent).
- `hcl.Expression` field type defers evaluation to a later step.
  To make variables or functions available in attribute expressions, pass an `*hcl.EvalContext` as the second arg to `DecodeBody`. For example, to expose the process ID as `pid`:
  HCL input:
  ```hcl
  name = "example-program (${pid})"
  ```
  Go side:
  ```go
  type Context struct {
  Pid string
  }
  ctx := gohcl.EvalContext(&Context{Pid: strconv.Itoa(os.Getpid())})
  moreDiags := gohcl.DecodeBody(f.Body, ctx, &c)
  ```
  - `gohcl.EvalContext` converts Go struct field names to HCL variable names automatically: `Pid` becomes `pid`, `ListenAddr` becomes `listen_addr`.
  - Methods on the struct become callable functions in HCL expressions.

### 2. hcldec (dynamic schema)

- Best for plugin systems where the schema is not known until the plugin is loaded.

```go
spec := hcldec.ObjectSpec{
  "io_mode": &hcldec.AttrSpec{Name: "io_mode", Type: cty.String},
  "service": &hcldec.BlockMapSpec{...},
}
val, diags := hcldec.Decode(body, spec, evalCtx)
```

- Spec types: `ObjectSpec`, `AttrSpec`, `BlockMapSpec`, `BlockListSpec`, etc.
- Each spec both validates structure and produces a result value.
- Returns `cty.Value` instead of native Go types.
- Can be mixed with gohcl: gohcl for known top-level, hcldec for plugin portions.

> `cty` is a type system independent of Go's, designed for cross-boundary value passing. Same reason Fowler advocates a semantic model independent of the host language.

### 3. Low-level API

- Decodes one nesting level at a time. Nested block bodies remain as `hcl.Body` for later decoding.
- All decoding mechanisms share the same `hcl.Body` type and can be freely mixed: gohcl for known top-level structure, low-level API for portions where more detailed source locations are needed.

`hcl.Body` methods:

- **`Content(schema)`**: closed-world decode. Anything not in the schema is an error.
- **`PartialContent(schema)`**: open-world decode. Returns matched content plus a remainder `hcl.Body` for unmatched elements to be decoded elsewhere.
- **`JustAttributes()`**: attributes-only mode, no schema needed. Useful when attribute names are unknown until runtime. Errors if blocks are present. In JSON syntax, all object properties are treated as attributes in this mode.
- **`MissingItemRange()`**: returns a source range pointing to where an absent required item should go, used for "missing required argument" diagnostics.

`Content` and `PartialContent` both take an `hcl.BodySchema` and return `hcl.BodyContent`:

```go
schema := &hcl.BodySchema{
  Attributes: []hcl.AttributeSchema{
    {Name: "io_mode", Required: false},
  },
  Blocks: []hcl.BlockHeaderSchema{
    {Type: "service", LabelNames: []string{"type", "name"}},
  },
}
content, moreDiags := body.Content(schema)
```

- `content.Attributes` is a `map[string]hcl.Attribute`. Attributes are uniquely named and unordered.
- `content.Blocks` is a `[]hcl.Block`. Blocks are ordered and may have multiple instances of the same type.
- Each block has its type, label values, and a nested `hcl.Body` for later decoding.
- Each attribute has an `hcl.Expression` for later evaluation.

`hcl.Expression` methods:

- **`Value(ctx)`**: evaluate the expression against an eval context, returns `cty.Value`.
- **`Variables()`**: lists all variable references in the expression without evaluating. Useful for static dependency analysis.
- **`Range()`**: source range for the entire expression. Use in diagnostics when the error is about the whole expression.
- **`StartRange()`**: source range for just the opening tokens of a complex expression (tuple, object). Use when the error is about the construct type, not its contents, to avoid an overwhelming source snippet.

```go
ctx := &hcl.EvalContext{
  Variables: map[string]cty.Value{
    "name": cty.StringVal("Ermintrude"),
  },
}
val, moreDiags := expr.Value(ctx)
diags = append(diags, moreDiags...)
```

> The most relevant approach for dboxide. Structural decoding first, expression evaluation later.

## Expression Evaluation

::: note
Input: `hcl.Expression` + `hcl.EvalContext` (variables, functions)
Output: `cty.Value`

- `gohcl` and `hcldec` both accept an eval context and evaluate expressions immediately during decode. The two phases are merged into one call.
- The low-level API keeps them separate: `Content` / `PartialContent` return `hcl.Expression` objects, and you call `expr.Value(ctx)` yourself later. This section describes that explicit, deferred step.
- Deferring evaluation gives you control over evaluation order, needed for interdependent blocks or gradual evaluation where not all values are available at the same time.
- `cty` unknown values act as typed placeholders, enabling type-checking before all values are resolved.
:::

::: note
`${}` is template interpolation syntax - only needed to embed an expression inside a string. Outside strings, expressions are written directly:

```hcl
count   = 2 + 3           # arithmetic expression
enabled = var.debug       # variable reference
message = "hello ${name}" # expression embedded in a string
```
:::

**Variables**

Variables are `cty.Value` entries in a `map[string]cty.Value`:

```go
ctx := &hcl.EvalContext{
  Variables: map[string]cty.Value{
    "name": cty.StringVal("Ermintrude"),
    "age":  cty.NumberIntVal(32),
  },
}
```

```hcl
message = "${name} is ${age} ${age == 1 ? "year" : "years"} old!"
```

Object values enable dotted attribute access in expressions:

```go
ctx := &hcl.EvalContext{
  Variables: map[string]cty.Value{
    "path": cty.ObjectVal(map[string]cty.Value{
      "root":   cty.StringVal(rootDir),
      "module": cty.StringVal(moduleDir),
    }),
  },
}
```

```hcl
source_file = "${path.module}/foo.txt"
```

**Functions**

Functions are defined in the `Functions` field and called by users in expressions:

```go
ctx := &hcl.EvalContext{
  Functions: map[string]function.Function{
    "upper":  stdlib.UpperFunc,
    "lower":  stdlib.LowerFunc,
    "strlen": stdlib.StrlenFunc,
  },
}
```

```hcl
message = "HELLO, ${upper(name)}!"
```

- The `cty` stdlib package provides standard string/list/math functions. Prefer these over reimplementing them per application.
- Some HCL native operators (e.g. arithmetic) also have stdlib function-shaped equivalents - these should generally not be exposed to avoid user confusion.

**Evaluation modes**

The eval context controls both error messages and JSON template behavior:

- `nil` context: variables and functions are "not available at all". In JSON syntax, strings are treated as plain literals, not templates.
- Non-nil context with nil maps: "not supported" errors. JSON strings parse templates but can still produce "not supported" errors.
- Populated maps: errors name the specific missing variable or function.

> The three levels let you tune how helpful the error messages are at each processing phase.

## Design Patterns for Complex Use Cases

The parsing and decoding mechanisms covered so far are sufficient for most applications. The patterns below address situations where configuration blocks cross-reference each other, where evaluation must happen across multiple processes or time points, or where different subsystems own different parts of the schema.

### Interdependent Blocks

**Problem**: Sometimes, blocks reference values defined in other blocks, so evaluation order matters (e.g. Terraform's `variable`, `locals`, and `resource` blocks all cross-reference each other).

> DBML also has cross-refs.

Steps:

1. Decode top-level structure only, without evaluating expressions.
  Use the low-level API or `gohcl` with `remain` fields to get one `hcl.Body` per block.
2. Call `Variables()` on **each attribute expression** to **detect which variables each block references**.
3. Build a dependency graph from these references.
4. Topological sort to determine evaluation order.
5. Evaluate each block in order, adding its result to the eval context before moving to the next.

Notes:

- Keep a `map[string]cty.Value` as your "results so far" store.
  - After evaluating each block, add its result to this map.
  - Before evaluating the next block, construct a fresh `hcl.EvalContext` from the current state of the map.
  - Do not try to mutate an existing `hcl.EvalContext` in place.
- `hcldec` is convenient here because `hcldec.Decode` returns a `cty.Value` directly, which can be inserted straight into the map.

  With `gohcl`, you get a Go struct and would have to manually convert it back to `cty.Value` to expose it as a variable for other blocks.

> A dependency network (Fowler ch. 7) applied to config evaluation.

### Gradual Evaluation (Distributed Systems)

**Problem**: The same configuration must be evaluated in multiple separate processes or at different points in time, and not all values are known upfront (e.g. Terraform: plan phase evaluates what it can, apply phase re-evaluates with the full set of values once remote APIs have returned IDs and other opaque values).

Two strategies:

- **Ahead-of-time evaluation**: evaluate all expressions fully in one subsystem, then pass only the resulting `cty.Value` constants to other subsystems.
  - Pros:
    - Simpler.
    - Works well for plugins where consistency matters more than flexibility.

    Both Nomad and Terraform use this for provider plugins: the plugin describes its schema, the core evaluates expressions on its behalf, and passes a fully-known value.
- **Gradual evaluation**: pass the original source files between subsystems and re-evaluate at each stage with an increasingly complete context.
  - Use `cty.UnknownVal(type)` as placeholders for values not yet available:
    ```go
    ctx := &hcl.EvalContext{
      Variables: map[string]cty.Value{
        "name": cty.UnknownVal(cty.String),
        "age":  cty.UnknownVal(cty.Number),
      },
    }
    val, moreDiags := expr.Value(ctx)
    ```
  - Unknown values propagate through expressions automatically: HCL performs type checking but returns another unknown when an input is unknown.
  - Each re-evaluation with more known values produces a more complete result.
  - Pass source files between subsystems (not intermediate values) to preserve source location information for diagnostics.

> Multi-pass parsing from Fowler ch. 3, made type-safe by `cty` unknown values.

### Partial Decoding with `remain`

**Problem**: Different subsystems or plugins own different parts of the configuration, and the top-level decoder cannot know the schema for plugin-specific blocks ahead of time.

- Decode only the known top-level structure, capturing unknown block bodies in `remain` fields.
- Pass each `remain` body to the plugin or subsystem that owns it, which then decodes it with its own schema and eval context.
- Terraform uses this to hand provider-specific resource blocks to provider plugins.

> Essentially Foreign Code (Fowler) applied to configuration files.

## Static Analysis

In most cases, only the result value of an expression matters.

**But sometimes the structure of the expression is more important than its value**.

For example:

- `depends_on` in Terraform **requires the user to name another object by reference**, not provide an object value.
  > This is important for DBML records, for example.
- HCL cannot evaluate `cloud_network.example` as a meaningful value in that context, but it can inspect the expression's syntax tree to extract the reference statically.

```hcl
depends_on = [
  cloud_network.example,
]
```

HCL provides **static analysis functions** in the `hcl` package. Each one imposes a structural requirement on the expression and returns a result if the expression conforms. They do not evaluate expressions.

These are applied after structural decoding, on the `hcl.Expression` objects you get back from attributes:

```go
// Decode structurally to get expressions
content, _ := body.Content(schema)
attr := content.Attributes["some_attr"]

// Instead of evaluating: attr.Expr.Value(ctx)
// Inspect structure statically:
result := hcl.ExprAsKeyword(attr.Expr)
```

**`ExprAsKeyword(expr) string`**

Interprets the expression as a single identifier. Returns empty string on failure.

```hcl
mode = async   # ExprAsKeyword returns "async"
mode = "async" # fails - string literal, not identifier
mode = foo.bar # fails - not a single keyword
```

```go
keyword := hcl.ExprAsKeyword(attr.Expr)
if keyword == "" {
  // not a keyword - produce a diagnostic
}
```

- `true`, `false`, `null` are accepted so the application can reuse them as application-level keywords.

**`AbsTraversalForExpr(expr) (Traversal, Diagnostics)`**

Accepts a variable name followed by zero or more attribute or index accesses with constant operands.

```hcl
ref = foo             # valid: root variable
ref = foo.bar         # valid: attribute access
ref = foo[0]          # valid: constant index
ref = foo[bar]        # invalid: non-constant index
```

```go
traversal, diags := hcl.AbsTraversalForExpr(attr.Expr)
if diags.HasErrors() { ... }
// traversal[0] is the root variable name
// traversal[1..] are the attribute/index steps
```

- `true`, `false`, `null` are accepted as root names.

**`RelTraversalForExpr(expr) (Traversal, Diagnostics)`**

Same as `AbsTraversalForExpr` but the first element is marked as an attribute of an implied outer object, not a root variable. Used when the traversal is relative to some context object.

**`ExprList(expr) ([]Expression, Diagnostics)`**

Requires a tuple constructor `[...]` and returns the element expressions. Terraform uses this to unwrap `depends_on`, then calls `AbsTraversalForExpr` on each element:

```hcl
depends_on = [
  cloud_network.example,
  cloud_subnet.other,
]
```

```go
elems, diags := hcl.ExprList(attr.Expr)
for _, elem := range elems {
  traversal, diags := hcl.AbsTraversalForExpr(elem)
  // use traversal to build dependency graph
}
```

**`ExprMap(expr) ([]KeyValuePair, Diagnostics)`**

Requires an object constructor `{...}` and returns key/value pairs for further static analysis.

```hcl
tags = {
  env  = production
  team = platform
}
```

```go
pairs, diags := hcl.ExprMap(attr.Expr)
for _, pair := range pairs {
  key := hcl.ExprAsKeyword(pair.Key)    // "env", "team"
  val := hcl.ExprAsKeyword(pair.Value)  // "production", "platform"
}
```

**`ExprCall(expr) (*FunctionCallExpr, Diagnostics)`**

Requires a function call expression and returns the function name and argument expressions.

```hcl
source = file("config.json")
```

```go
call, diags := hcl.ExprCall(attr.Expr)
// call.Name == "file"
// call.Args[0] is the expression for "config.json"
```

**`Variables()` on `hcl.Expression`**

Lists all variable references in an expression without evaluating it. Treated as a fundamental feature because it is needed for both static validation and interdependent block resolution.

```go
vars := attr.Expr.Variables()
// vars is []hcl.Traversal - each traversal is one variable reference
for _, v := range vars {
  fmt.Println(v.RootName()) // "var", "local", "cloud_network", etc.
}
```

> Dependency analysis, name resolution, and reference validation can all be done entirely statically using these functions, without ever evaluating expressions.
