---
_type: Concept
_label: "API Boundary"
status: 'mastered'
tags:
  - 'architecture'
---

An explicit interface layer that separates different components of a system. By making boundaries opaque (hiding implementation details), you can refactor internal code without breaking external consumers. Example: the separation between the `hir` analysis layer and the `ide` layer in `rust-analyzer`.
