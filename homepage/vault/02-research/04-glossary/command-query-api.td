---
_type: Concept
_label: "Command-Query API"
status: 'learning'
tags:
  - 'dsl classification'
depends_on:
  - fref("02-research/04-glossary/fluent-interface.td")
---

A regular, non-fluent API where you call methods to get things done. Fowler uses this term to contrast with fluent interfaces: a command-query API is imperative wiring (`idle.addTransition(doorClosed, active)`), while a fluent interface reads like a declaration (`idle.transition(doorClosed).to(active)`).
