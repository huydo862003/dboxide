---
_type: Concept
_label: "Interning"
status: 'mastered'
tags:
  - 'incremental compilation'
---

An optimization technique that stores only one copy of duplicate data. In rust-analyzer, identical tokens (like multiple occurrences of `1` in `1 + 1`) share the same allocated token, reducing memory usage.
