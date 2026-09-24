---
_type: Concept
_label: "Lattice"
status: 'learning'
tags:
  - 'type system'
  - 'mathematics'
---

A partially ordered set where every pair of elements has a least upper bound (join) and a greatest lower bound (meet).

- **Join**: the most general value that both elements satisfy. `string | int` generalizes both `string` and `int`.
- **Meet**: the most specific value that satisfies both constraints simultaneously. `string & =~"^foo"` is the meet of `string` and the regex constraint.

In CUE, all types and values occupy a single lattice:

- The top element (`_`) matches anything.
- The bottom element (`_|_`) is the error value - nothing satisfies it. Reached when two constraints are incompatible.
- Concrete values like `"hello"` sit below their type (`string`) because they are more specific.
- Unifying two values means computing their meet. Incompatible constraints produce bottom, which is a type error.

> This is why CUE can collapse type checking and validation into one operation: both are just meet computation in the same lattice.
