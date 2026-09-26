---
_label: "dboxide DSL"
_icon: icon.document
---

dboxide keywords are case-insensitive:

| Category | Keywords |
| --- | --- |
| Definition | `type`, `fn`, `get` |
| Block | `namespace`, `constraints` |
| Module | `use`, `reuse`, `from`, `as` |
| Expression | `match`, `forall`, `exists`, `of` |
| Reference | `self`, `super` |

::: note
The language core is just keywords and operators. Everything else - `Table`, `Ref`, `decl`, `ref`, `[element]`, `[field]`, `string`, `bool` - is provided by the `metatype` module via `use * from 'metatype'`.
:::

## Tokens

### Literals

| Kind | Examples |
| --- | --- |
| String | `'hello'`, `"world"` |
| Integer | `42`, `-1` |
| Float | `3.14`, `-0.5` |
| Boolean | `true`, `false` |
| Null | `null` |
| Expression | `` `now()` `` - opaque SQL, passed through verbatim |

### Identifiers

Bare identifiers follow `[a-zA-Z_][a-zA-Z0-9_]*`. Quoted identifiers allow spaces and special characters: `"not null"`, `"on use"`.

### Operators

An operator is any sequence of non-alphanumeric symbols. Custom operators are possible. Alphanumeric names are functions, not operators. Operators are resolved by operand types (typed dispatch).

```
operator_char := [.@|?=<>:!\-+*/%^&~]
operator      := operator_char+
```

Built-in operators used by the `metatype` module:

| Operator | Name | Prec | Assoc |
| --- | --- | --- | --- |
| `.` | Dot | 12 | left |
| `@` | At (member of) | 12 | left |
| `!` | Not (prefix) / Not (binary) | 11 / 7 | - / none |
| `*` | Deref (prefix) / Multiply | 11 / 10 | - / left |
| `/` | Divide | 10 | left |
| `%` | Modulo | 10 | left |
| `+` | Positive (prefix) / Add | 11 / 9 | - / left |
| `-` `?-` `-?` `?-?` | Negate (prefix) / Subtract / One-to-one and nullable variants (ref) | 11 / 9 / 7 | - / left / none |
| `<:` | Subtype | 8 | left |
| `<=` | Less than or equal | 7 | none |
| `>=` | Greater than or equal | 7 | none |
| `<` `?<` `<?` `?<?` | Less than / Many-to-one and nullable variants (ref, binary/unary) | 7 | none |
| `>` `?>` `>?` `?>?` | Greater than / One-to-many and nullable variants (ref, binary/unary) | 7 | none |
| `<>` `?<>` `<>?` `?<>?` | Many-to-many and nullable variants (ref, binary/unary) | 7 | none |
| `==` | Equal | 7 | none |
| `!=` | Not equal | 7 | none |
| `^` | Xor | 6 | left |
| `&&` | And | 5 | left |
| `||` | Or | 4 | left |
| `++` | Concat | 9 | left |
| `..` | Range (prefix/infix/postfix) | 9 | none |
| `??` | Coalesce | 3 | right |
| `~` | Mixin (prefix) | 11 | - |
| `=>` | Arrow (closure) | 2 | right |
| `->` `<-` | Dep direction (upstream/downstream) | 7 | none |
| `|` | Pipe (type union) | - | - |
| `?` | Question (nullable) | - | - |
| `[]` | Brackets (array) | - | - |

### Punctuation

| Token | Role |
| --- | --- |
| `{` `}` | Block delimiters |
| `(` `)` | Group / composite key delimiters |
| `[` `]` | Annotation delimiters |
| `,` | Separator |
| `:` | Type annotation / key-value separator |

## Basic Structure

An element's body is composed of **fields** and **sub-elements**.

- **Fields** appear as single lines, with arguments parsed positionally and settings in brackets.
- **Sub-elements** appear as nested blocks with their own keyword and body.

```
Table users {
  // fields: one line each
  id integer [pk]
  name varchar [not null]

  // sub-element: nested block
  indexes {
    id [pk]
  }
}
```

This composition is recursive: a sub-element can itself contain fields and further sub-elements.

## Object System

Every construct in dboxide is an object. Every object creates a namespace. `super` is a built-in name in every namespace, pointing to the parent namespace.

### Named and Anonymous Objects

Objects are either **named** or **anonymous**.

- **Named** objects carry an explicit name in their syntax. The name is declared in the enclosing namespace and can be referenced elsewhere.

```
Table users { ... }          // "users" is declared in the schema namespace
Enum status { ... }          // "status" is declared in the schema namespace
```

- **Anonymous** objects have no name of their own. Their identity is anchored to the objects they reference.

```
Ref: users.id > posts.user_id    // no name; identified by the endpoints it connects
records { 1, 'Alice' }           // no name; identified by the table it belongs to
```

Anonymous objects may optionally carry names (`Ref order_fk: ...`, `Dep order_staging { ... }`), but the name is not required for the construct to be valid.

### Schemas

Schemas are implicit top-level namespaces: they are not declared explicitly but are introduced by qualifying an element name with a prefix. All elements belong to the `public` schema by default, and `public` may be omitted even in qualified names.

```
Table users { ... }               // belongs to public schema (default)
public.Table users { ... }        // same as above, public is explicit
auth.User { ... }                 // belongs to public.auth schema
public.auth.User { ... }          // same, public is explicit
```

A schema may only appear as a namespace prefix on top-level elements. It cannot be introduced inside another element.

### Namespace Access

Since every object creates a namespace, two access patterns are needed.

| Operator | Form | Resolves |
| --- | --- | --- |
| `.` | `x.y` | Namespace traversal - looks up a named child `y` within `x` (like `schema.table.column`) |
| `@` | `x@y` | Member access - reads the value of field `y` defined on `x` |

```
users.id        // namespace lookup: Column named "id" within "users"
users@columns   // member access: the Column[] list stored on the "users" instance
Table@columns   // member access on the type: the field definition itself
```

Built-in namespace references:

- `self`: the current namespace (the instance being validated).
- `super`: the parent namespace. Since `super` is a name in every namespace, chaining works: `super.super` for grandparent.

```
self@pk              // pk field of current instance
super@columns        // all columns on the enclosing Table
super.super@columns  // grandparent's columns field
```

## Type Definitions

A type definition introduces a named structure with typed fields. The role annotation after the name determines how instances are written in the surface syntax.

### Generic Types

Type definitions support type parameters for generic programming.

```
type Pair[A, B] {
  first A
  second B
}

type Map[K, V] {
  entries Pair[K, V][]
}
```

Type parameters are resolved at instantiation:

```
columns Map[string, Column]     // K=string, V=Column
settings Pair[string, any]      // A=string, B=any
```

### Roles

Two roles are available.

- `[element]`: a construct with its own named block body. Can appear at the top level or nested inside a parent element as a sub-element (e.g. `indexes { }`, `checks { }`).
- `[field]`: a construct written as a single line inside a parent block, with arguments parsed positionally or by keyword.

An optional `[on use: expand]` annotation may be added to an `[element]` type. When an instance of that type is imported, all `ref` fields are dereferenced and the namespaces of the referenced instances are injected into the importer - effectively expanding the container into its members.

```
type TableGroup [element, on use: expand] {
  tables ref[Table][]
}
// use { tablegroup auth_core }
//   -> dereferences tables -> gets each Table instance
//   -> injects each Table's namespace into the importer
```

::: note
Schemas behave as `[on use: expand]` by definition. Importing a schema injects all elements within that schema's namespace into the importer, without needing explicit annotation.
:::

Types may nest: a type defined inside another type's body is scoped to that parent.

```
type Table [element] {
  type Column  [field]   { ... }   // line inside Table block
  type Index   [element] { ... }   // sub-element: indexes { ... }
  type Check   [element] { ... }   // sub-element: checks { ... }
}
```

### Type Body Sections

A type body has three optional sections:

- **Fields** (default body): data the type holds, with syntax annotations.
- **`namespace {}`**: selects which fields are exposed via `.` traversal. Supports bare names (expose a field) and `get name(): Type { expr }` (computed getters).
- **`constraints {}`**: named constraints that must hold for every instance.

```
type Table [element] {
  /* fields */
  name qdecl[Table] [label, unique]
  columns Column[]
  partials ref[TablePartial][]
  indexes Index[]

  /* namespace: what `.` traversal sees */
  namespace {
    columns                        // expose columns field directly
    get all_columns(): Column[] {  // computed getter: merge partials
      columns ++ partials.flatMap((p) => p@columns)
    }
  }

  /* validation */
  constraints {
    no_dup_columns: () => columns.map((c) => c@name).is_unique()
  }
}
```

### Field Resolution

When a type contains multiple `[field]` types, each line in the surface syntax is matched against the field type definitions in source order. The first type whose form and annotations match the input wins. More specific field types should be defined before more general ones.

### Fields

Each field has a name, a type, and syntax annotations.

Annotations control how the field maps to the surface syntax:

::: note
All syntax annotations (`[arg]`, `[csv]`, `[label]`, `[attribute]`, `[setting]`) are primitives from the `metatype` module.
:::

#### Argument Position

| Annotation | Syntax form | Meaning |
| --- | --- | --- |
| `[arg: n]` | `field T [arg: 0]` | Binds field to positional arg at index `n` |
| `[arg: n..]` | `field T [arg: 1..]` | Binds field to all args from index `n` (variadic) |
| `[csv]` | `field T [arg: 0, csv]` | Arg is a comma-separated list parsed inline |

```
type Column [field] {
  name decl[Column] [arg: 0, label, unique]   // "id"      <- positional arg 0
  type Type         [arg: 1]          // "integer" <- positional arg 1
}
// surface:  id integer [pk]
```

#### Label

| Annotation | Syntactic slot |
| --- | --- |
| `[label]` | The primary name, written directly after the keyword |
| `[label alias]` | The alias, written after `as` |

```
type Table [element] {
  name  qdecl[Table] [label]         // Table <name> { ... }
  alias qdecl[Table] [label alias]          // Table <name> as <alias> { ... }
}
// surface:  Table users as u { ... }
```

#### Structure

- `[attribute]`: the field is written as a bare keyword in the syntax.
  - On a `[field]` type: the keyword appears inline on the field line, after positional args (e.g. `pk` in `id integer pk`).
  - On an `[element]` type: the keyword appears as a `key: value` entry in the body block.
- `[setting]`: the field is written inside the `[...]` bracket (e.g. `[pk, not null, default: 0]`).

A field may carry both `[attribute, setting]` if it can be written in either position.

```
type Column [field] {
  type Type  [attribute]   // written as inline keyword: id integer
  pk   bool? [setting]     // written inside brackets: id integer [pk]
  name decl[Column] [arg: 0, unique]
}
```

#### Rest Fields

`...name` captures all undeclared body attributes into a typed map field.

```
...metadata: Map[string, any] [attribute]
```

#### Annotation Values as Implicit Strings

Bare words and multi-word phrases inside `[...]` annotations are implicitly string literals. Quoting is optional unless the value contains characters that would be ambiguous.

```
[setting]              // same as ["setting"]
[on use: expand]       // same as ["on use": "expand"]
[arg: 0]               // numeric value, not a string
[label alias]          // same as ["label alias"]
```

### Declarations and References

::: note
`decl`, `qdecl`, `ref`, and `qref` are primitives from the `metatype` module.
:::

A declaration introduces a name into a namespace. A reference resolves a declared name by searching the current namespace, then enclosing namespaces, then imported namespaces, in that order.

#### Declarations

Two declaration forms are available, differing in whether the declared name also exposes a namespace.

- `decl[T]`: declares a name in the namespace of the enclosing type `T`. The name is not visible outside it.
- `qdecl[T]`: like `decl[T]`, but additionally exposes the container's own namespace under the declared name. This allows references of the form `name.member` to resolve into the container.

```
type Column [field] {
  name decl[Column] [arg: 0, label, unique]   // "id" declared in Column's namespace
}

type Table [element] {
  name qdecl[Table] [label, unique]   // declares "users"; exposes Table's namespace
}
// Table users { id integer }  ->  "users.id" resolves into Table's namespace
```

#### References

Two reference forms are available, differing in whether the reference traverses a qualified namespace to reach its target.

- `ref[T]`: a reference to a declaration of type `T`, resolved by searching enclosing namespaces in order.
- `qref[T]`: like `ref[T]`, but the reference is qualified - it follows the named namespace exposed by `qdecl[T]` to reach the declaration (e.g. `schema.table.column`).

```
type Index [element] {
  columns ref[Column][]    // resolves column names in the enclosing Table's namespace
}
```

## Type System

### Principles

dboxide follows the "types are values" principle: types and values inhabit the same lattice. A type is a constraint on the set of values it accepts, and a concrete value is a maximally specific type.

- `any` is the top of the lattice - the most general type, accepting all values.
- A literal like `"btree"` is both a value and a type that accepts exactly that value.
- More specific types sit lower in the lattice. Concrete values sit at the bottom.

### Type Compatibility

A type `A` is compatible with type `B` (written `A <: B`) if every value accepted by `A` is also accepted by `B`. Compatibility follows the lattice structure:

- Every type is compatible with `any` (the top).
- A literal type is compatible with its base type (`"btree" <: string`).
- A disjunction `A | B` is compatible with any type that accepts both `A` and `B`.
- `null` is compatible with any nullable type (`T?`).

```
"btree" <: string            // literal is subtype of its base
"btree" <: "btree" | "hash"  // literal is subtype of a disjunction containing it
integer <: any               // every type is subtype of any
```

### Type Aliases

A type alias binds a name to a type expression. It is transparent - interchangeable with the expression it names - and introduces no new structure.

```
type Name = <type-expr>
```

```
type IndexMethod = "btree" | "hash"   // disjunction
type NullableInt = integer?           // shorthand for integer | null
type Identifier  = string             // rename for clarity
```

## Expressions

dboxide does not support loops or conditional statements. Instead, expressions are built from operators, function calls, closures, pattern matching, and quantifiers.

### Closures

```
(params) => body
```

```
(c) => interpret(c@type)       // single parameter
() => null ^ "not null"        // no parameters
```

### Ternary

```
condition ? then_expr : else_expr
```

### Pattern Matching

```
match expr {
  pattern1 => result1
  pattern2 => result2
  _ => default
}
```

### Quantifiers

High-level assertions over collections.

- `forall`: the assertion must hold for every element.
- `exists`: the assertion must hold for at least one element.

```
forall c of columns { c@type <: ValidType }
exists c of columns { c@pk == true }
```

## Function Definitions

Functions are defined with `fn`, require typed signatures, and use block bodies. `:` separates parameters from return type. `=>` is exclusively the closure arrow.

```
fn function_name(param: Type, ...): ReturnType { body }
```

```
fn typeof(v: any): Type { ... }
fn interpret(t: Type): Type { ... }
```

### Operator Definitions

Custom operators are symbol-only sequences defined with `fn operator<symbol>`. Operators are regular functions with typed dispatch - the same symbol can have different implementations for different operand types.

Built-in operators have well-defined precedence and associativity. Custom operators always have a fixed default precedence and left associativity - like PostgreSQL, where user-defined operators share a single precedence level to keep parsing predictable.

| Prec | Assoc | Operators |
| --- | --- | --- |
| 12 | left | `.` `@` |
| 11 | left | `!` `~` (prefix) |
| 10 | left | `*` `/` `%` |
| 9 | left | `+` `-` |
| 8 | left | `<:` |
| 7 | none | `<` `>` `<=` `>=` `==` `!=` `->` `<-` |
| 6 | left | `^` |
| 5 | left | `&&` |
| 4 | left | `||` |
| 3 | right | `??` |
| 2 | right | `=>` (closure) |
| 1 | left | custom operators (default) |

```
fn operator~>(a: bool, b: bool): bool { !a || b }   // custom op, prec 1, left assoc
```

## Constraints

Constraints are validation rules over a type instance's field values. They appear inside a `constraints {}` block and are checked after all fields are resolved. A violated constraint is a static error.

Each constraint is a named entry of the form `name: closure`. The name is used in error messages; the closure must return true for every instance of the enclosing type.

Bare and quoted names inside constraint expressions refer to fields of the enclosing type (e.g. `pk` is the `pk` field, `"not null"` is the `"not null"` field).

```
type Column [field] {
  null     bool? [setting]
  "not null" bool? [setting]
  pk       bool? [setting]

  constraints {
    nullability: () => null ^ "not null"
    pk_not_null: () => !pk || "not null"
  }
}
```

Each constraint value is a closure using `(params) => body` syntax:

```
type_check: () => typeof(values) <: columns.map((c) => interpret(c@type))
```

## Module System

> See also: fref("03-high-level-design/01-original-dbml-language-spec.td") - Module System

A dboxide schema may be split across multiple files. All files are merged into one logical schema before analysis. `use` and `reuse` are top-level only.

**Import all** imports everything a file or built-in module exports.

```
use * from './other.dbml'
use * from 'metatype'     // built-in module by name
```

**Selective import** imports only the specified elements, avoiding unintended name conflicts.

```
use {
  table users
  enum status
} from './other.dbml'
```

**Aliases** rename an import to avoid conflicts. Only the alias name is accessible afterward.

```
use {
  table users as auth_users
} from './auth'
```

**reuse** works like `use` but also re-exports the imports to any file that imports the current file. Use it to expose schema elements without forcing consumers to know the internal folder structure.

```
// common/index.dbml
reuse * from './users'    // users is visible to importers of common/index
use * from './internal'   // internal stays private to common/index
```

::: note
`use` is not transitive. Elements imported via `use` are not visible to files that later import the current file. Use `reuse` to pass them through.
:::

## Metatype Constructs

The `metatype` module provides all built-in types, operators, functions, and annotations. All constructs are qualified under `metatype` (e.g. `metatype.string`, `metatype.interpret()`). Import with `use * from 'metatype'` to use them unqualified.

### Type Constructors

| Construct | Meaning |
| --- | --- |
| `decl[T]` | Declares a name in the namespace of enclosing type `T`; not visible outside |
| `qdecl[T]` | Like `decl[T]`, but exposes the container's namespace under the declared name |
| `ref[T]` | Reference to a declaration of type `T` |
| `qref[T]` | Qualified reference, traversing a namespace exposed by `qdecl[T]` |
| `Partial[T]` | All fields of `T`, all optional - for mixin/template injection |
| `Map[K, V]` | Key-value map from `K` to `V` |

### Built-in Types

All types are provided by the `metatype` module. A literal like `"hello"` has type `metatype.string`.

| Type | Meaning |
| --- | --- |
| `string` | String values |
| `integer` | Integer values |
| `float` | Floating-point values |
| `bool` | Boolean values (`true`, `false`) |
| `null` | Null value |
| `color` | Color literals (`#ff0000`) |
| `any` | Top type; accepts any value |
| `ostring` | Opaque string; passed through verbatim without interpretation |

### Syntax Annotations

| Annotation | Meaning |
| --- | --- |
| `[arg: n]` | Binds field to positional arg at index `n` |
| `[arg: n..]` | Binds field to all args from index `n` onwards (variadic) |
| `[csv]` | Argument is a comma-separated list parsed inline |
| `[label]` | Primary name slot, written after the keyword |
| `[label alias]` | Alias slot, written after `as` |
| `[attribute]` | Field is written as a bare keyword in the syntax |
| `[setting]` | Field is written inside `[...]` brackets |
| `[unique]` | Field value is unique within the enclosing namespace |

### Role Annotations

| Annotation | Meaning |
| --- | --- |
| `[element]` | Block construct with its own body |
| `[field]` | Line-based construct inside a parent block |
| `[on use: expand]` | On import, deref all `ref` fields and inject referenced namespaces |

### Functions

| Function | Signature | Meaning |
| --- | --- | --- |
| `interpret` | `fn interpret(t: Type): Type` | Type-level function: maps a field type to its value domain |
| `typeof` | `fn typeof(v: any): Type` | Returns the runtime type of value `v` |
| `operator<:` | `fn operator<:(a: Type, b: Type): bool` | Subtype check |
| `operator??` | `fn operator??(a: any?, b: any): any` | Coalesce: LHS if set, otherwise RHS |
| `operator^` | `fn operator^(a: bool, b: bool): bool` | Xor: exactly one is true |
| `operator++` | `fn operator++(a: T[], b: T[]): T[]` | Collection concatenation |

Methods are defined with `fn` inside a type body:

```
type T[] {
  fn map(f: (T) => U): U[] { ... }
  fn flatMap(f: (T) => U[]): U[] { ... }
  fn is_unique(): bool { ... }
}
```

### Operators

| Operator | Signature | Meaning |
| --- | --- | --- |
| `|` | `fn (a: Type) | (b: Type): Type` | Type union (disjunction) |
| `?` | postfix on `Type` | Shorthand for `T | null` |
| `[]` | postfix on `Type` | Zero or more values of `T` |
| `.` | `fn (x: any) . (y: name): any` | Namespace traversal |
| `@` | `fn (x: any) @ (y: name): any` | Member access |
| `<:` | `fn (a: Type) <: (b: Type): bool` | Subtype check |
| `^` | `fn (a: bool) ^ (b: bool): bool` | Xor: exactly one is true |
| `??` | `fn (a: any?) ?? (b: any): any` | Coalesce: LHS if set, otherwise RHS |

## Formalizing DBML Elements

```dbml
use * from 'metatype'

type RefAction = "cascade" | "restrict" | "set null" | "set default" | "no action"

/* Ref operators - typed dispatch on qref[Column][]
 * Column < Column  -> many-to-one RefBinary
 * Column > Column  -> one-to-many RefBinary
 * Column - Column  -> one-to-one RefBinary
 * Column <> Column -> many-to-many RefBinary
 * ? on either side marks nullable: ?<, <?, ?<?, etc.
 */

/* binary: Ref [name]: table.col op table.col [settings] */
fn operator<(from: qref[Column][], to: qref[Column][]): RefBinary { RefBinary { from, to, cardinality: "<"    } }
fn operator>(from: qref[Column][], to: qref[Column][]): RefBinary { RefBinary { from, to, cardinality: ">"    } }
fn operator-(from: qref[Column][], to: qref[Column][]): RefBinary { RefBinary { from, to, cardinality: "-"    } }
fn operator<>(from: qref[Column][], to: qref[Column][]): RefBinary { RefBinary { from, to, cardinality: "<>"  } }
/* + all nullable binary variants: ?<, <?, ?<?, ?>, >?, ?>?, ?-, -?, ?-?, ?<>, <>?, ?<>? */

/* unary: [ref: > users.id] - from is implicitly the enclosing column */
fn operator<(to: qref[Column][]): RefUnary { RefUnary { to, cardinality: "<"    } }
fn operator>(to: qref[Column][]): RefUnary { RefUnary { to, cardinality: ">"    } }
fn operator-(to: qref[Column][]): RefUnary { RefUnary { to, cardinality: "-"    } }
fn operator<>(to: qref[Column][]): RefUnary { RefUnary { to, cardinality: "<>"  } }
/* + all nullable unary variants */

/* Dep operators - typed dispatch on qref[Column | Table]
 * Table|Column -> Table|Column  generates a DepBinary (upstream to downstream)
 * Table|Column <- Table|Column  generates a DepBinary (downstream to upstream)
 */

/* binary: Dep: raw_orders -> stg_orders */
fn operator->(from: qref[Column | Table], to: qref[Column | Table]): DepBinary { DepBinary { from, direction: "->", to } }
fn operator<-(to: qref[Column | Table], from: qref[Column | Table]): DepBinary { DepBinary { from, direction: "<-", to } }

/* unary: [dep: -> stg_orders] - from/to is implicitly the enclosing element */
fn operator->(to: qref[Column | Table]): DepUnary { DepUnary { direction: "->", target: to } }
fn operator<-(from: qref[Column | Table]): DepUnary { DepUnary { direction: "<-", target: from } }

fn operator~(p: ref[TablePartial]): TablePartialInjection { TablePartialInjection { partial: p } }

/* Project */

type Project [element] {
  name decl[Project] [label, unique]
  database_type string? [attribute]
  note string? [attribute]
}

/* Enum */

type Enum [element] {
  name qdecl[Enum] [label, unique]
  variants Variant[]

  namespace {
    variants
  }

  type Variant [field] {
    name decl[Variant] [arg: 0, label, unique]
    note string? [setting]
  }
}

/* RefBinary - result of binary ref operators (a.col > b.col) */

type RefBinary [field] {
  from qref[Column][] [arg: 0]
  cardinality string [arg: 1]
  to qref[Column][] [arg: 2]
  delete RefAction? [setting]
  update RefAction? [setting]
  color color? [setting]
  inactive bool? [setting]
}

/* Ref [name]: table.col op table.col [settings]
 * Ref [name] {
 *   table.col op table.col [settings]
 * }
 */

type Ref [element] {
  name decl[Ref]? [label]
  endpoints RefBinary[]
}

/* RefUnary - result of unary ref operators ([ref: > users.id]) */

type RefUnary [field] {
  cardinality string [arg: 0]
  to qref[Column][] [arg: 1]
  delete RefAction? [setting]
  update RefAction? [setting]
}

/* DepBinary - result of binary dep operators (a -> b) */

type DepBinary [field] {
  from qref[Column | Table] [arg: 0]
  direction "->" | "<-" [arg: 1]
  to qref[Column | Table] [arg: 2]
}

/* Dep [name]: a -> b
 * Dep [name] {
 *   a -> b
 *   c -> d
 *   note: 'Staging transformation'
 * }
 */

type Dep [element] {
  name decl[Dep]? [label]
  lines DepBinary[]
  color color? [setting]
  note string? [setting]
  ...metadata: Map[string, any] [attribute]
}

/* DepUnary - result of unary dep operators ([dep: -> stg_orders]) */

type DepUnary [field] {
  direction "->" | "<-" [arg: 0]
  target qref[Column | Table] [arg: 1]
}

/* TablePartial name [settings] {
 *   col_name col_type [settings]
 *   indexes { ... }
 * }
 */

type TablePartial [element] {
  name decl[TablePartial] [label, unique]
  headercolor color? [setting, attribute]
  note string? [setting, attribute]
  columns Table.Column[]
  indexes Table.Index[]

  namespace {
    columns
  }
}

/* TableGroup name [settings] {
 *   table_name
 * }
 */

type TableGroup [element, on use: expand] {
  name decl[TableGroup] [label, unique]
  tables ref[Table][] [field: 0..]
  note string? [setting]
  color color? [setting]
}

/* Note: 'string'
 * Note { '''multi-line''' }
 */

type Note [element] {
  content string [arg: 0]
}

/* Note name [settings] { 'text' } */

type StickyNote [element] {
  name decl[StickyNote] [label, unique]
  content string [arg: 0]
  color (color | "none")? [setting]
}

/* Metadata Table users { owner: 'scott' }
 * Metadata Column users.email { pii: 'true' }
 */

type Metadata [element] {
  target qref[Table | Table.Column | Enum | TableGroup] [label]
  ...metadata: Map[string, any] [attribute]
}

/* DiagramView name {
 *   Tables { * }
 *   Tables { users, orders }
 *   Notes { * }
 *   TableGroups { billing }
 *   Schemas { public }
 * }
 */

type DiagramView [element] {
  name decl[DiagramView] [label, unique]

  type Tables [element] {
    items (ref[Table] | "*")[] [field: 0..]
  }

  type Notes [element] {
    items (ref[StickyNote] | "*")[] [field: 0..]
  }

  type TableGroups [element] {
    items (ref[TableGroup] | "*")[] [field: 0..]
  }

  type Schemas [element] {
    items (string | "*")[] [field: 0..]
  }
}

/* Table */

type Table [element] {
  name qdecl[Table] [label, unique]
  alias qdecl[Table] [label alias]
  headercolor color? [setting, attribute]
  note string? [setting, attribute]
  partials TablePartialInjection[]
  indexes Index[]
  checks Check[]
  columns Column[]
  deps DepUnary[] [setting: dep]
  ...metadata: Map[string, any] [attribute]

  namespace {
    columns
    get all_columns(): Column[] {
      columns ++ partials.flatMap((p) => p@partial@columns)
    }
  }

  type TablePartialInjection [field] {
    partial public.TablePartialInjection [arg: 0]
  }

  /* <name> <type> */
  type Column [field] {
    name decl[Column] [arg: 0, label, unique]
    type Type [arg: 1, attribute]
    unique bool? [attribute, setting]
    pk bool? [attribute, setting]
    null bool? [setting]
    "not null" bool? [setting]
    increment bool? [setting]
    default interpret(type)? [setting]
    note string? [setting]
    refs RefUnary[] [setting: ref]
    deps DepUnary[] [setting: dep]
    checks ostring[]

    constraints {
      nullability: () => null ^ "not null"
      pk_not_null: () => !pk || "not null"
    }
  }

  type Index [element] {
    columns (ref[Column] | ostring)[] [field: 0.., attribute]
    pk bool? [setting]
    unique bool? [setting]
    type "btree" | "hash" [setting]
    name string? [setting]
    note string? [setting]
  }

  /* checks {
   *   `expression` [settings]
   * }
   */
  type Check [element] {
    expression ostring [field: 0, attribute]
    name string? [setting]
  }

  /* records (col1, col2)? {
   *   value, value, ...
   * }
   */
  type Records [element] {
    columns ref[Column][]? [field: 0.., label, attribute]
    row Row[]

    type Row [field] {
      values any[] [field: 0, csv]

      constraints {
        type_check: () => typeof(values) <: (columns ?? super.super@columns).map((c) => interpret(c@type))
        // columns             -> Records@columns (explicit column list, if provided)
        // super               -> enclosing Records instance
        // super.super         -> enclosing Table instance (namespace traversal)
        // super.super@columns -> all columns on the Table
      }
    }
  }
}
```
