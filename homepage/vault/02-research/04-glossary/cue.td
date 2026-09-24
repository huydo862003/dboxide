---
_type: Concept
_label: "CUE (Configure, Unify, Execute)"
status: 'learning'
tags:
  - 'dsl design'
  - 'type system'
---

A constraint-based configuration language where types and values are unified into a single lattice.

- Every **value** is also a **type**.
- Every **type** is also a **constraint**.

Constraining a value and type-checking it are the same operation. Validation is not a separate pass but is intrinsic to the type system itself.

Developed by Marcel van Lohuizen (formerly Google).

> Contrasted with HCL: HCL separates structural decoding from schema validation as distinct phases. CUE collapses them into one by making the schema a set of constraints that values must satisfy to exist at all.
