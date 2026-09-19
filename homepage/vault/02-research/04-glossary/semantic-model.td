---
_type: Concept
_label: "Semantic Model"
status: 'learning'
tags:
  - 'dsl design'
depends_on:
  - fref("02-research/04-glossary/domain-model.td")
  - fref("02-research/04-glossary/adaptive-model.td")
---

The in-memory model that a DSL populates. Syntax is what the DSL looks like (captured by the grammar). Semantics is what it means (captured by the model). The DSL is a thin facade over the semantic model. All representations of the same DSL (XML, custom syntax, internal DSL) produce the same configuration of objects in the semantic model. Fowler advocates that a semantic model is a vital part of a well-designed DSL because it separates parsing from semantics.
