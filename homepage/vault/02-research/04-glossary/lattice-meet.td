---
_type: Concept
_label: "Lattice Meet"
status: 'learning'
tags:
  - 'type system'
  - 'mathematics'
depends_on:
  - fref("02-research/04-glossary/lattice.td")
---

The greatest lower bound of two elements in a lattice. The meet of A and B is the most specific element that satisfies both A and B simultaneously.

In CUE, meet is the `&` operator. `string & =~"^foo"` is the meet of `string` and the regex constraint - more specific than either alone. Incompatible constraints produce bottom.
