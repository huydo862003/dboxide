---
_type: Concept
_label: "Reentrancy"
status: 'learning'
tags:
  - 'type system'
  - 'linguistics'
depends_on:
  - fref("02-research/04-glossary/feature-structure.td")
---

A shared substructure in a feature structure where two different paths from the root point to the same node. The node is not duplicated - both paths reference the same value.

```
[ subject: [1] [ name: string ] ]
[ agent:   [1]                  ]
```

Tag `[1]` marks reentrancy: `subject` and `agent` point to the same feature structure. Changing one changes the other.

Reentrancy is what makes feature structures graphs rather than trees. It is used in linguistics to express agreement constraints (e.g., subject and verb must share the same number feature).
