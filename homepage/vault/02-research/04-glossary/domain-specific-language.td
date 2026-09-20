---
_type: Concept
_label: "Domain-Specific Language (DSL)"
status: 'learning'
tags:
  - 'dsl classification'
depends_on:
  - fref("02-research/04-glossary/semantic-model.td")
---

A computer programming language of limited expressiveness focused on a particular domain (Martin Fowler).

Four key elements:

- **Computer programming language**: used by humans to instruct a computer. Designed for human readability but must be executable.
- **Language nature**: DSL is a programming language. Expressiveness comes not just from individual expressions but from how they compose together. This is the "fluency" that separates a DSL from a bag of API calls.
- **Limited expressiveness**: supports only the features needed for its domain. No general-purpose control structures or abstractions. You cannot build an entire system in a DSL.
- **Domain focus**: a consequence of limited expressiveness. A limited language is only useful if it has a clear focus on a small domain.
