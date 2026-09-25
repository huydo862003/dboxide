---
_type: Concept
_label: "Lattice Join"
status: 'learning'
tags:
  - 'type system'
  - 'mathematics'
depends_on:
  - fref("02-research/04-glossary/lattice.td")
---

The least upper bound of two elements in a lattice. The join of A and B is the most general element that both A and B are instances of.

In CUE, join is the `|` operator. `string | int` is the join of `string` and `int` - the most general type that includes both.
