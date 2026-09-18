---
_type: Concept
_label: "DST"
status: 'mastered'
tags:
  - 'type system'
---

Dynamically Sized Type. A Rust type whose size is not known at compile time. Used in rust-analyzer's GreenNode implementation to store node data and all children in a single heap allocation: `[RefCount | Kind | TextLen | N_Children | Child_1 | ... | Child_N]`.
