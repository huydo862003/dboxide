---
_type: Concept
_label: "Object Scoping"
status: 'learning'
tags:
  - 'internal dsl'
  - 'dsl pattern'
depends_on:
  - fref("02-research/04-glossary/function-sequence.td")
  - fref("02-research/04-glossary/expression-builder.td")
---

Place the DSL script in a subclass of the Expression Builder so that all function calls resolve as instance methods. This avoids global functions and global parsing state. The builder instance holds all parsing data as instance fields, and users can extend the DSL by subclassing. Fowler's default recommendation for Function Sequence.
