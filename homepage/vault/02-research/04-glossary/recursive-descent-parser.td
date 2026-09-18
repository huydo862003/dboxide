---
_type: Concept
_label: "Recursive Descent Parser"
status: 'mastered'
tags:
  - 'parsing'
---

A top-down parsing technique where each grammar rule maps to a function that calls other parsing functions recursively. Makes the parser code easy to understand and debug since it directly mirrors the grammar structure. Used by rust-analyzer for most syntactic constructs.
