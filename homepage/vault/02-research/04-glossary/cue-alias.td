---
_type: Concept
_label: "Alias (CUE)"
status: 'learning'
tags:
  - 'type system'
  - 'dsl'
depends_on:
  - fref("02-research/04-glossary/feature-structure.td")
---

A named binding to a field value within a struct, written `A=field: constraint`. The alias `A` is a relative reference that resolves contextually wherever the struct is used, not at the point of definition.

```cue
#Config: {
  M=metadata: #Metadata
  data: M
}
```

`M` refers to `metadata` as it exists in the final unified struct, including any concrete values. This differs from `#Config.metadata`, which bakes in the reference at declaration time and resolves only to the abstract constraint.
