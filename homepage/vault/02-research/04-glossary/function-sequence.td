---
_type: Concept
_label: "Function Sequence"
status: 'learning'
tags:
  - 'internal dsl'
  - 'dsl pattern'
depends_on:
  - fref("02-research/04-glossary/context-variable.td")
  - fref("02-research/04-glossary/object-scoping.td")
---

A DSL function combination pattern using bare function calls in sequence: `processor(); cores(2); speed(2500);`. Simple to implement and calls execute in written order, but requires global functions and context variables to track which object is being built. Object Scoping is the recommended fix for the global state problem.
