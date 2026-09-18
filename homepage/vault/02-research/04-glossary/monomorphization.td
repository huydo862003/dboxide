---
_type: Concept
_label: "Monomorphization"
status: 'mastered'
tags:
  - 'type system'
---

The process where Rust's compiler generates separate copies of generic functions for each concrete type used. Can lead to "monomorphization bloat" where excessive use of generics at system boundaries causes long compile times. Mitigated by restricting generics or using dynamic dispatch at boundaries.
