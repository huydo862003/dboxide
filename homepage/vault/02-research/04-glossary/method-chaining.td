---
_type: Concept
_label: "Method Chaining"
status: 'learning'
tags:
  - 'internal dsl'
  - 'dsl pattern'
depends_on:
  - fref("02-research/04-glossary/fluent-interface.td")
  - fref("02-research/04-glossary/expression-builder.td")
---

A DSL function combination pattern where each method returns `this` (or a related builder) to allow chaining: `.processor().cores(2).speed(2500)`. Methods are scoped to the Expression Builder. Optional elements are easy to handle (just skip the call), but it requires context variables and breaks command-query separation.
