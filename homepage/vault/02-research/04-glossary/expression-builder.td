---
_type: Concept
_label: "Expression Builder"
status: 'learning'
tags:
  - 'dsl'
  - 'internal dsl'
depends_on:
  - fref("02-research/04-glossary/fluent-interface.td")
  - fref("02-research/04-glossary/semantic-model.td")
---

A layer of objects whose sole job is to provide the fluent interface for an internal DSL and populate the semantic model underneath. DSL scripts invoke methods on the expression builder, which translates those calls into model operations.

Without this explicit layer, DSL code and model code easily get mixed together.
