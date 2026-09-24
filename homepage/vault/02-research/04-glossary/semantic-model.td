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

The in-memory model that a DSL populates.

- The grammar captures syntax.
- The semantic model captures meaning.

- All representations of the same DSL (XML, custom syntax, internal DSL) produce the same object graph in the semantic model.
- Separating parsing from the model means parsing code does not embed domain logic.
