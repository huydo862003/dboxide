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

## Architecture of DSL Processing

```
DSL script --> parse --> semantic model --> (optional) generate --> target code
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

::: details On "parsing" an internal DSL
"Parsing" an internal DSL means executing it. The input is function calls instead of a text stream, but the idea is the same: arrange them into a structure that populates the model.

The common definition of internal DSL (written in the host language) is usually right but not always. A JRuby DSL inside a Java app is still internal. The true distinction: an internal DSL is **executed**; an external DSL is **read** as a data structure.
:::

Executing the model directly is simpler and usually the right choice. Code generation is optional.

::: details When to use code generation
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

Fowler discourages this: the syntax tree is tied to the DSL syntax, so it couples processing to syntax. Changing the DSL means changing the processing logic too.
