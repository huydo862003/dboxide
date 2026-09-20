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
Building the DSL is usually not the hard work but building the model is.
:::

> Basically, the DSL is just an API for interacting with the model - it's user-facing.
