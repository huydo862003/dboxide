---
_type: Book
_label: "Language Workbenches"
author: "Martin Fowler"
date: "2010"
description: "Tools that help you build DSLs and their editing environments together"
isbn: "978-0321712943"
url: "https://martinfowler.com/books/dsl.html"
tags:
  - "dsl"
  - "language workbench"
  - "tooling"
---

Language workbenches are tools that help you build DSLs and provide IDE-level tool support for them. Someone writing a DSL script gets the same experience a programmer gets with a modern IDE: syntax highlighting, completion, error checking.

## Three Elements

A language workbench defines three aspects of a DSL:

- **Semantic model schema**: the data structure of the model, defined using a meta-model rather than host language classes.
- **DSL editing environment**: a rich editing experience for writing DSL scripts, through source editing or projectional editing.
- **Semantic model behavior**: what the DSL script does, usually via code generation from the model.

## Schema Definition Languages and Meta-Models

When you build a DSL by hand, the schema is the host language's class definitions (`State`, `Event`, `Transition`). Language workbenches instead define the schema using a **meta-model**: a model whose instances define the structure of another model.

```java
// Meta-model: defines what the state machine model can contain
MClass state = new MClass("State");
MClass event = new MClass("Event");
MClass transition = new MClass("Transition");
transition.addField(new MField("source", state));
transition.addField(new MField("target", state));
transition.addField(new MField("trigger", event));
```

- The meta-model is itself a model. Its schema can be defined by another meta-model, which eventually bottoms out at a self-describing model. This is a **bootstrapped workbench**.
- A **schema definition language** is a DSL for defining meta-models. It defines the structure (entities and relationships), not the concrete syntax of any particular DSL.
- A **grammar** defines the concrete syntax of a textual DSL. A schema definition language defines the structure of the semantic model. They are complementary, not the same thing.
- Schemas can also include **structural constraints** (validation rules on what makes a valid model instance), similar to Design by Contract invariants.

## Source vs Projectional Editing

**Source-based editing**: the program is text. You edit the text, and the tool parses it into a model. This is how all mainstream programming works.

**Projectional editing**: the tool stores the semantic model directly. When you edit, the tool **projects** the model into an editable representation (text, diagram, table, form). You never touch a text file. Microsoft Access is a familiar example.

- Pros of projectional editing:
  - Multiple representations of the same model (text, diagram, table) that stay in sync.
  - The editor can constrain input to only valid constructs, giving a tighter feedback loop.
  - Refactorings operate on the model, not on text, making them safer.
- Cons of projectional editing:
  - Vendor lock-in. Code written in one workbench cannot be exported to another.
  - No access to text-based tools (grep, diff, version control, sed).
  - Source code management (branching, merging, diffing) relies entirely on text today.

Modern IDEs are a hybrid: **model-assisted source editing**. You edit source text, but the IDE continuously parses it into a model to provide projectional features (completion, refactoring, error highlighting).

## Illustrative Programming

Projectional editing enables **illustrative programming**: instead of reading a general program definition, you see concrete illustrations of what the program does with specific inputs, updated live as you edit. Spreadsheets are the most popular example.

- The program is hidden (formulas in cells). The illustrations (computed values) are front and center.
- This makes it accessible to lay programmers who do not think of themselves as programmers.
- The downside is that program structure gets de-emphasized, leading to unmaintainable programs at scale.

## Tools

- **JetBrains MPS**: projectional editing with structured text representation. Open source. Targets programmer productivity.
- **Xtext/Langium**: source-based editing built on Eclipse/LSP. Uses ANTLR as parser back-end. Provides model-assisted source editing for DSL scripts.
- **MetaEdit (MetaCase)**: focused on graphical/tabular projections. Oldest released tool.
- **Intentional Workbench**: most sophisticated. Supports all projection types (text, tables, diagrams). Rich meta-modeling. Not widely released.

## Language Workbenches vs CASE Tools

CASE tools (1990s) look superficially similar: central model, meta-models, diagrammatic editing. The key difference is that CASE tools did not let you define your own language. Language workbenches give you full control over the language definition.

## Should You Use One?

- High potential but high risk. Tools are still maturing and lock-in is significant.
- One mitigation: treat the workbench as a parser only, not a full DSL environment. Build the semantic model in your host language. Use the workbench to define the editing experience and generate Model Aware code against your semantic model. If the workbench fails, only the parser is locked in.
