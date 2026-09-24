---
_type: Concept
_label: "Command-Query API"
status: 'learning'
tags:
  - 'dsl classification'
depends_on:
  - fref("02-research/04-glossary/fluent-interface.td")
---

A regular, non-fluent API where you call methods to get things done. Fowler uses this term to contrast with fluent interfaces.

- Command-query API: imperative wiring. `idle.addTransition(doorClosed, active)`
- Fluent interface: reads like a declaration. `idle.transition(doorClosed).to(active)`
