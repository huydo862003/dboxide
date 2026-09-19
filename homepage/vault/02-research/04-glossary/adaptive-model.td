---
_type: Concept
_label: "Adaptive Model"
status: 'learning'
tags:
  - 'dsl design'
depends_on:
  - fref("02-research/04-glossary/semantic-model.td")
  - fref("02-research/04-glossary/domain-model.td")
---

A model whose population effectively acts as the program for the system. Changing behavior means altering the objects and their wiring, not changing code. This blurs the distinction between code and data. A state machine is a classic example: the states and transitions ARE the program. A DSL makes adaptive models usable by providing a readable way to define the configuration.
