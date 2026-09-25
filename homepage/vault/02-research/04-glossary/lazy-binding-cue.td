---
_type: Concept
_label: "Lazy Binding (CUE)"
status: 'learning'
tags:
  - 'type system'
  - 'evaluation'
depends_on:
  - fref("02-research/04-glossary/feature-structure.td")
---

In CUE, field definitions do not need to appear before the fields they reference. All fields are in scope for all other fields, resolved as a fixed point over the whole struct. This is lazy binding: name resolution is deferred until the full structure is assembled.

This is analogous to `letrec` (mutually recursive bindings) in Lisp - every binding is simultaneously in scope for every other binding.

```cue
a: b + 1
b: 5
```

`a` references `b` before `b` is defined, but this is valid. CUE resolves both once the struct is complete.

- This is not optional laziness. All references must still resolve to a defined field in the final closed structure. Undefined references are errors.
- Lazy binding in CUE is a consequence of the TFS/subsumption definition, not a design decision. Subsumption requires comparing complete feature structures, so constraints must be fully accumulated before any resolution occurs.
