---
_type: Concept
_label: "GreenNode"
status: 'mastered'
tags:
  - 'syntax tree'
depends_on:
  - fref("02-research/04-glossary/lossless-syntax-tree.td")
---

The immutable storage layer in the Red-Green tree architecture. Stores the actual syntax tree data in a space-efficient, persistent structure that can be shared across threads via `Arc`. Contains only `kind: SyntaxKind`, `text_len: usize`, and children, with no position information or parent pointers. Optimizations include DST layout, tagged pointers, and token interning.
