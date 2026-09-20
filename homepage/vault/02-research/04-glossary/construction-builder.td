---
_type: Concept
_label: "Construction Builder"
status: 'learning'
tags:
  - 'parsing'
  - 'dsl'
depends_on:
  - fref("02-research/04-glossary/semantic-model.td")
---

A mutable intermediate object used during parsing when the target semantic model object is read-only after construction. It accumulates data gradually as the DSL is parsed, then creates the real object once all data is available.
