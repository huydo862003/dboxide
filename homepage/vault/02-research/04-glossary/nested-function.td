---
_type: Concept
_label: "Nested Function"
status: 'learning'
tags:
  - 'internal dsl'
  - 'dsl pattern'
---

A DSL function combination pattern where function calls are nested as arguments: `computer(processor(cores(2), speed(2500)), disk(size(150)))`. The hierarchy is explicit in the code and no context variables are needed because arguments are evaluated before the parent call. Downsides: inside-out evaluation order, positional arguments lack names, and parentheses add noise.
