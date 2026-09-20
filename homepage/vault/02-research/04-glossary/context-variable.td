---
_type: Concept
_label: "Context Variable"
status: 'learning'
tags:
  - 'parsing'
  - 'dsl'
---

A variable that tracks the current parsing context (e.g. "which state are we currently building") when it cannot be inferred from the tree structure alone. Treat as a smell: they make parsing code harder to follow, like mutable globals in regular code.
