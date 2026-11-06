---
_type: Concept
_label: "Adaptive Model"
status: 'learning'
tags:
  - 'dsl design'
depends_on:
  - fref("./semantic-model.td")
  - fref("./domain-model.td")
---

A model whose data acts as the program. Changing behavior means altering objects and their wiring, not changing code.

- A state machine is a classic example: the states and transitions are the program.
- A DSL makes adaptive models usable by providing a readable way to define the configuration.
