---
_type: Concept
_label: "GreenNode"
status: 'mastered'
tags:
  - 'syntax tree'
depends_on:
  - fref("02-research/04-glossary/lossless-syntax-tree.td")
---

The immutable storage layer in the Red-Green tree architecture. Shareable across threads via `Arc`. Contains only `kind: SyntaxKind`, `text_len: usize`, and children. No position information or parent pointers.

Optimizations:

- DST layout
- Tagged pointers
- Token interning
