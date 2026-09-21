---
_type: Book
_label: "Using Domain-Specific Languages"
author: "Martin Fowler"
date: "2010"
description: "Formal definition of DSL & their pros/cons"
isbn: "978-0321712943"
url: "https://martinfowler.com/books/dsl.html"
tags:
  - "dsl"
  - "language design"
  - "patterns"
---

This chapter formally defines what a DSL is and discusses their benefits and problems.

## Definition

::: info
**Domain-specific language** (noun): a computer programming language of limited expressiveness focused on a particular domain.
:::

Four key elements to the definition:

- **Computer programming language**: DSL used by humans to instruct a computer.
  - Designed for human readability but must be executable.
- **Language nature**: **DSL is a programming language**.
  - Expressiveness comes not just from individual expressions but from how they compose together.
  - This is the "fluency" that separates a DSL from a bag of API calls.
- **Limited expressiveness**: supports only the features needed for its domain. No general-purpose control structures or abstractions. You cannot build an entire system in a DSL.
- **Domain focus**: a consequence of limited expressiveness. A limited language is only useful if it has a clear focus on a small domain.

::: note
Domain focus comes last because it is a consequence, not a cause. Many people define DSL literally as "a language for a specific domain", but the limitation in expressiveness is what actually makes it a DSL.
:::

### Three Categories of DSLs

- **External DSL**: a language separate from the host application. Usually has a custom syntax, but can use XML or another carrier syntax. Parsed by the host using text parsing techniques.
  - Examples: regular expressions, SQL, Awk, XML configuration files.
- **Internal DSL**: a particular way of using a general-purpose language. Valid code in the host language, but uses only a subset in a particular style. Should feel like a custom language.
  - Classic examples: Lisp, Ruby (Rails, RSpec), Kotlin (Gradle DSL).
- **Language workbench**: a specialized IDE for defining and building DSLs. Defines both the structure of the DSL and a custom editing environment for writing scripts.

::: warning
These three communities rarely overlap. People experienced in internal DSLs often do not know how to build external DSLs, and vice versa.
:::

### DSL as Abstraction Front-End

Another way to look at a DSL: it is a front-end to a library or framework, providing a different manipulation style than the command-query API.

- The library/framework is the semantic model.
- The command-query API is the imperative way to manipulate it.
- The DSL is the declarative way to manipulate it.

::: tip
The model brings its own benefits independent of the DSL: it removes duplication, provides a reusable abstraction, and encapsulates behavior. The DSL is a thin layer on top. Building the model is usually the hard work.
:::

> Basically, the DSL is just an API for interacting with the model - it's user-facing.

## Boundaries of DSLs

DSL boundaries are blurry. The real tests are **language nature** and **limited expressiveness**.

### Internal DSL vs Command-Query API

- A command-query API defines the vocabulary; an internal DSL adds a grammar.
- Methods in an internal DSL only make sense within a larger expression. `.to(unlockedPanel)` is meaningless alone but reads naturally inside `.transition(lightOn).to(unlockedPanel)`.
- Limited expressiveness in an internal DSL comes from self-restraint: you use only a narrow slice of the host language. Piers Cawley calls this a **pidgin** use of the host language.

### External DSL vs General-Purpose Language

- A language can have domain focus and still be general-purpose. **R** targets statistics but is Turing-complete, so Fowler would not call it a DSL.
- The clearest indicator of a DSL is that it is not Turing-complete: no variables, no subroutines, no general control flow.
- **XSLT** is a corner case: it has domain focus but is Turing-complete. Whether it counts as a DSL depends on usage. The same language can sit on either side of the line.

### Config Files

- A config file is a DSL when it is the primary way humans interact with that part of the system. If it is only a machine exchange format, it is a serialization format.
- A series of property assignments (`color = blue`) lacks fluency and does not count.
- In practice, machine-generated XML often becomes the primary human-editing mechanism anyway. **XML becomes a DSL by accident.**

### Language Workbench vs Form-Based App

- A language workbench produces a DSL when the editing experience has the fluency and grammar of a language. Most form-based UIs do not achieve that, even if they manipulate the same underlying model.

### Human Jargon

- Starbucks ordering has domain focus, limited expressiveness, and grammar, but it is a human language, not a computer language. Fowler reserves "DSL" for computer languages and uses **domain language** for the human equivalent.

## Fragmentary vs Stand-alone DSLs

- A **stand-alone DSL** is a self-contained script, typically a single file. Someone familiar with the DSL but not the host language can read and understand it on its own.
- A **fragmentary DSL** appears as small snippets embedded within host language code. You cannot follow it without understanding the surrounding host code.

Examples:

- **Stand-alone**: the state machine custom syntax like in ${fref("02-research/02-resources/05-dsl/domain-specific-languages/01-an-introductory-example.td")}.
- **Fragmentary (external)**: regular expressions and SQL embedded in a larger program.
- **Fragmentary (internal)**: mock object expectation grammars in unit tests. Annotations that add metadata to host code elements.

::: note
Annotations are a popular Java feature for building internal fragmentary DSLs. They attach metadata to host code elements like classes and fields (e.g. `@Entity`, `@Table`), and a framework reads them at runtime. Because they always need a host code element to sit on, you cannot have a file that is purely annotations. This makes them only suitable for fragmentary DSLs.
:::

The same DSL can be used in both forms. SQL is a good example: it appears as embedded statements in application code and also as stand-alone query files.

## Why Use a DSL?

DSLs are a specific tool for specific conditions, not a paradigm shift.

::: note
Always separate the benefits of the **model** from the benefits of the **DSL** (see [DSL as Abstraction Front-End](#dsl-as-abstraction-front-end)). It is a common mistake to conflate the two.
:::

- **Clarity**: A DSL makes intent clearer than a command-query API. Limited expressiveness makes it harder to say wrong things and easier to spot mistakes.
- **Reduced surface area**: The DSL only exposes what is actually used, cutting down what developers need to learn.
  - Wrapping a poor third-party API amplifies this further.
- **Domain expert communication**: Domain experts can read DSL scripts and spot mistakes without knowing the host language.
  - The gain is **reading**, not writing. A DSL designed with domain experts has value even if never implemented.
- **Execution context**: A DSL lets you express rules comfortably and generate code for the target environment (SQL, C, JavaScript).
  - Scripts also live in version control, unlike rules buried in a database.
- **Declarative models**: Some problems fit state machines, dependency networks, or production rules better than imperative code.
  - The model provides the computation.
  - The DSL makes it easy to populate and read.

::: warning
The **COBOL fallacy**: "business people can now write the rules themselves, without programmers". It rarely holds.
:::

## Problems with DSLs

Do not use a DSL when none of the benefits apply, or when the cost outweighs the benefit.

- **Language cacophony**: The concern that many languages make a project harder to learn. But DSLs are far simpler than general-purpose languages.
  - **The real question** is how much harder it is to learn a DSL versus learning the underlying model directly.
  - Since a DSL is designed to make the model easier to understand, the incremental learning cost is usually small or negative.
- **Cost of building**: A DSL adds code to write and maintain. If a command-query API does the job, there is no value adding a DSL on top. External DSLs in particular add parsers that many developers find intimidating.
  - The cost should be weighed against the cost of the model alone, not zero - a complicated enough problem to need a DSL almost certainly needs a model too.
- **Ghetto language**: Building whole systems in an in-house language traps you. A DSL should stay narrow. Watch for features creeping in - conditionals today, loops tomorrow, and suddenly you have an accidental general-purpose language.
- **Blinkered abstraction**: A comfortable abstraction can make you reluctant to change it. DSLs make this worse because they make the abstraction more comfortable to work in. Domain experts are especially prone to this once they get used to a DSL. Treat any DSL as evolving, not finished.

## DSL Lifecycle

Two approaches to starting a DSL:

- **Model-first**: build the framework and command-query API, then layer the DSL on top once you see the need. Common when the DSL was not planned upfront.
- **DSL-first**: sketch the DSL with domain experts before any implementation. Write example scripts in the form you want, then build the model and DSL together incrementally. Good for establishing a shared language early.

When growing a DSL on top of an existing model:

- **Language-seeded**: treat the model as a black box. Sketch pseudo-DSL for existing use cases, then implement scenario by scenario with minimal changes to the model.
- **Model-seeded**: add fluent methods to the model first, then gradually extract them into a DSL. More gradual, lower upfront cost, better suited to internal DSLs.

Either way, the critical step is separating the model from the configuration code. The DSL comes after that separation.

## What Makes a Good DSL Design?

The goal is clarity for the reader: your target audience (programmer or domain expert) should understand DSL sentences quickly.

- Use the jargon of the domain. If users know the terms, they should see them in the DSL and in the model.
- Follow conventions of the host environment. If your team writes Java, use `//` for comments and `{}` for structure.
- Iterate and test ideas on your target audience. Be prepared to try and discard many alternatives.
- Do not try to make the DSL read like natural language. Attempts at natural language syntax (like AppleScript) add syntactic sugar that obscures semantics. A DSL is a programming language: it should feel like programming, with the terseness and precision that implies.
