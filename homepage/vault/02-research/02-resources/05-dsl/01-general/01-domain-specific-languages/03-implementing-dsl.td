---
_type: Book
_label: "Implementing a DSL"
author: "Martin Fowler"
date: "2010"
description: "Common techniques for building DSLs: architecture, semantic model, parsing"
isbn: "978-0321712943"
url: "https://martinfowler.com/books/dsl.html"
tags:
  - "dsl"
  - "language design"
  - "patterns"
---

This chapter covers implementation techniques common to both internal and external DSLs.

::: note
Recap:

1. DSL parsing produces AST (either explicitly or implicitly).
2. The AST maps to a populated semantic model.
:::

## Architecture of DSL Processing

```
DSL script --parse-> semantic model --(optional) generate-> target code
```

The **semantic model** is where all the important behavior lives. The DSL's only job is to populate it via a parsing step. Once populated, you either execute the model directly or generate code from it.

- The model is a normal object model. It can be populated and run directly via its command-query API, independent of any DSL.
- It is usually a subset of the application's domain model. Not every part of the domain model needs a DSL, and a DSL can also be used for things outside the domain model.

::: note
An **object model** is a set of live instances wired together at runtime, each with data and behavior. The state machine at runtime is one: a `StateMachine` pointing to `State` instances, each pointing to `Transition` instances.

The semantic model does not have to be this rich. A plain data structure works too, but it holds no behavior - a framework would have to interpret it externally.
:::

::: note
The semantic model is not the same as an **abstract syntax tree (AST)**. An AST captures syntactic structure. The semantic model captures meaning.
:::

::: details Why keep the semantic model separate?
- Isolate semantic bugs from parsing bugs.
- Test the model directly via the command-query API, no parser needed.
- Support multiple DSLs over the same model without duplicating behavior.
- Evolve the model and DSL independently.

> The separation mirrors presentation/domain model in enterprise software. A DSL is essentially another form of user interface.
:::

Both internal and external DSLs produce the same semantic model. The difference is only in parsing.

- **External DSL**: parser reads text and populates the model.
- **Internal DSL**: host language handles the text. An **Expression Builder** layer provides the fluent interface and populates the model. Without this explicit layer, DSL and model code easily get mixed up.

::: details What "parsing" means for an internal DSL
"Parsing" an internal DSL means executing it. The input is function calls instead of a text stream, but the idea is the same: arrange them into a structure that populates the model.

The common definition of internal DSL (written in the host language) is usually right but not always. A JRuby DSL inside a Java app is still internal. The true distinction: an internal DSL is **executed**; an external DSL is **read** as a data structure.
:::

Executing the model directly is simpler and usually the right choice. Code generation is optional.

::: details When code generation is worth it
Use it when the target environment has limited language choices (embedded hardware, SQL). Parse in a comfortable language, generate C or SQL for the target.

Even then, keep a runnable semantic model. You can test without generating code and catch errors before generation. The semantic model also decouples the code generator from the parser.

Code generation also helps when a rich semantic model feels like "magic" to developers. Generated code makes behavior explicit.
:::

## Parser Workings

Any script can be read as a **syntax tree**: nodes are constructs, children are their parts. The tree is a more useful representation than raw text because you can walk and transform it.

::: note
The syntax tree is often not a real data structure. It lives implicitly on the call stack, forming and pruning as the parser walks the input. You only ever see the branch currently being processed.

This is similar to **SAX** (Simple API for XML): an event-based XML parser that fires events ("opening tag", "text", "closing tag") as it reads top to bottom, never holding the full tree in memory. A call-stack-based parser works the same way: one node at a time, no full tree.

Even so, it is a useful mental model.

How the implicit tree is formed in an internal DSL:

- **Nested Function**: nesting via arguments - `state("idle", transition("doorClosed", "active"))`.
- **Method Chaining**: nesting via chained calls - `.transition(doorClosed).to(active)`.
- **Function Sequence + Context Variables**: no nesting at all; a shared mutable context tracks the current position.

External DSLs produce a more explicit tree, sometimes materialized as a real data structure.
:::

Some approaches execute or generate code directly from the syntax tree, effectively using it as a semantic model.

Fowler discourages this because the syntax tree is tied to the DSL syntax, coupling any processing changes to syntax changes.

## Grammars, Syntax, and Semantics

A **grammar** is a set of production rules that describe how a text stream is turned into a syntax tree.

```
additionStatement := number '+' number
```

This tells the parser that `5 + 3` is an addition statement. Rules reference each other to form a full grammar.

- A language can have multiple valid grammars. There is no single "the grammar" for a language. Each grammar defines a different syntax tree structure for the same text.
- A grammar defines **syntax** only: how text maps to a tree. It says nothing about **semantics**: what the text means. `5 + 3` could mean `8` or `"53"` depending on context. Same syntax, different semantics.

With a semantic model, semantics is defined by how the syntax tree populates the model. Two expressions with different syntax but the same resulting model structure have the same semantics.

::: note
Internal DSLs involve two parsing passes and thus two grammars:

1. The host language parser reads the DSL script and produces executable instructions.
2. As those instructions execute, the DSL's implicit syntax tree forms on the call stack.

The DSL grammar only applies in the second pass.
:::

## Parsing Data

The parser only sees one branch of the syntax tree at a time.

References that cross branches (a command defined in one place, used in another) require extra data structures.

- **Symbol Table**: a dictionary mapping identifiers to their objects.
  1. When `unlockDoor D1UL` is parsed, the command is stored under `"unlockDoor"`.
  2. Later, `actions {unlockDoor}` looks it up to wire the relationship.
- **Context Variable**: tracks the current parsing context (e.g. "which state are we building") when it cannot be inferred from the tree.
  - Treat as a smell: they make parsing harder to follow, like mutable globals in regular code.
  - Similar to the `>>` ambiguity in C++: whether `vector<vector<int>>` closes one or two template arguments depends on context. Early compilers required `vector<vector<int> >` because they had no such tracking.
- **Construction Builder**: a mutable intermediate object that accumulates data for a semantic model object that is read-only after construction. Once all data is gathered, the real object is created.

Parsing can also be split into two phases:

1. First read the DSL into intermediate data
2. Then walk that data to populate the semantic model.

## Macros

Macros come in two flavors:

- **Textual macros**: simple text substitution. A macro processor replaces `MEDIUM_SHADE` with `#FFB595` before the file is processed. Can take parameters, e.g. C preprocessor's `sqr(x)` -> `x * x`. Fallen out of favor due to awkward edge cases.
- **Syntactic macros**: substitution on syntactically valid expressions, transforming one kind of expression into another.
  - Lisp is the canonical example.
  - C++ templates are a more familiar one.
  These are core to internal DSLs in Lisp, but only available in languages that support them.

## Testing DSLs

- **Semantic model tests**: populate the model directly via its command-query API, no parser involved.
- **Parser tests**: feed small DSL fragments and assert the correct model structure. Build a method that walks two model instances and collects all differences at once.
- **Script tests**: parse a DSL script, populate the model, run it, and check the output. Integration tests for the full pipeline.

::: note
Negative input tests are as important as positive input tests.
:::

::: tip
Add assertions inside the semantic model itself so it fails immediately with a clear error when put into an invalid state, rather than letting the caller discover it later.
:::

## Error Handling

Error handling has two locations: the parser and the semantic model.

- **Syntactic errors** naturally live in the parser. Some are handled for free (host language errors in an internal DSL, grammar errors in a parser generator).
- **Semantic errors** can go in either place. The model is the right place for checking well-formedness rules because all information is structured and available there.
  > It's like semantic errors are both checked at compile time and runtime.

The problem with model-only errors: no link back to the DSL script location (not even a line number).

Three strategies for semantic errors:

- **Detection in the parser**: has script context but works at the syntax tree level, making rules harder to write and easier to duplicate.
- **Push script info into the model**: add line numbers to model objects. Makes the model more complicated.
- **Model detects, parser reports**: parse a chunk, populate the model, then ask the model to check for errors. The parser adds script context to whatever the model finds. Separates syntactic knowledge (parser) from semantic knowledge (model).

::: note
Divide error handling into **initiation** (parser), **detection** (model), and **reporting** (both: model supplies the semantics, parser adds the script context).
- **Initiation**: the parser decides *when* to trigger error checking. After parsing a chunk and populating the model, the parser calls something like `model.validate()`.
- **Detection**: the model checks its own well-formedness rules and finds what is wrong.
- **Reporting**: both collaborate. The model says what the error is. The parser adds where in the script it came from.
:::

## Migrating DSLs

A DSL definition is a published interface. When it changes, existing scripts may break. Two migration strategies:

- **Incremental migration**: for each change to the DSL, create a migration script that transforms old scripts to the new version. Keep changes as small as possible. Ten small migrations are easier to write and chain than one big one.
- **Model-based migration**: keep parsers for each released version, all populating the same semantic model. To migrate a script, parse it with the old parser, then generate a new-version script from the model. Downside: non-semantic content like comments can be lost.

::: tip
Add a mandatory version statement to DSL scripts. Hard to retrofit later, but makes it easy to detect out-of-date scripts and trigger migrations automatically.
:::
