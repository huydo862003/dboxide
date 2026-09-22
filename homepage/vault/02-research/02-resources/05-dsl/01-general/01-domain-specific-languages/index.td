---
_type: Book
_label: "Domain-Specific Languages"
author: "Martin Fowler"
date: "2010"
description: "A comprehensive guide to designing, implementing, and using domain-specific languages. Covers both internal and external DSLs with practical patterns and examples."
isbn: "978-0321712943"
url: "https://martinfowler.com/books/dsl.html"
tags:
  - "dsl"
  - "language design"
  - "patterns"
---

Martin Fowler's definitive guide on domain-specific languages. Covers the full spectrum from internal DSLs (embedded in a host language) to external DSLs (with their own parser), along with a catalog of DSL patterns.

We research this book for two aspects of dboxide's DSL: the **structural DSL** (data description and validation) and the **logic DSL** (composable constraints and rules).

This book covers:

- When and why to use DSLs.
  - Tradeoffs involved when using DSLs.
- Two categories of DSLs:
  - Internal DSLs (embedded in a host language).
  - External DSLs (custom parser)
- The semantic model as **the core artifact** that DSL syntax populates.
- A catalog of ~50 implementation patterns for both internal and external DSLs.
- Alternative computational models that can be more suitable to solve problems in some domains:
  - Decision tables
  - State machines
  - Dependency networks
  - Production rules
- Code generation strategies from semantic models.
- Language workbenches (source editing vs projectional editing).

## Motivation for the Book

By mid-2000s:

- Java/C# dominated enterprise software. And they were thought to be the one true future language.
- However, people started realizing that some tasks can be more easily expressed in XBL: Important logic that didn't fit those languages ended up in XML config files.
- XML gave you custom vocabulary and hierarchy but the syntax was too noisy.

Then Ruby on Rails appeared:

- It popularized a more fluent approach to library design, making interacting with a library feel like programming in a specialized language.
- This thinking goes back to Lisp. Even Java/C# saw fluent interfaces become more popular (JMock, Hamcrest).

Fowler saw that people were picking the wrong DSL technique because they didn't know the alternatives.

## Why DSLs matter

Two reasons:

1. **Productivity**: A well-chosen DSL makes complicated code easier to understand, thus improving productivity for everyone working with it.
2. **Communication with domain experts**: DSLs can act as a common text that is both executable software and something domain experts can read to understand how their ideas are represented in a system. This is harder to achieve, but the gain is much broader because it helps unclog one of the worst bottlenecks in software development: communication between programmers and their customers.

## What The Book Cover

- What a DSL is.
- When a DSL is useful.
- How a DSL differs from a framework or library.
- How to build external and internal DSLs.
- Code generation strategies.
- A brief overview of language workbenches (promising but unproven at the time of writing).

Out-of-scope:

- DSL construction in functional languages (ML/Haskell) is not covered.
- Diagnostics and error handling are barely touched, despite being one of the hardest parts of compiler writing.
- The alternative computational models section is shorter than Fowler would have liked.

::: tip Reading guideline
This book has 2 parts:
- The first part (~150 pages) is a narrative designed to be read cover to cover.
- The second part is a reference catalog of ~50 patterns designed to be dipped into on demand.

The reference entries are self-standing so you can look up a specific pattern without rereading the narrative.
:::

::: note Pattern Structure
Each pattern has:
- Name
- Intent (1-2 sentences)
- Sketch (visual or code)
- A short motivating summary
- "How it works"
- "When to use it"
- Examples

Examples are deliberately simple and focused. They show the pattern, not a production-ready implementation.
:::

::: note The Reference Section
The reference section is a series of topic, each of with is usually a common solution (a pattern) to a recurring problem.
:::
