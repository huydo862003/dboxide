---
_type: Concept
_label: "Production Rule System"
status: 'learning'
tags:
  - 'computational model'
  - 'dsl'
depends_on:
  - fref("02-research/04-glossary/adaptive-model.td")
  - fref("02-research/04-glossary/decision-table.td")
---

Individual rules each with a condition and an action. The engine decides evaluation order.

- Chaining: firing one rule can trigger others. Rules stay independent, but emergent behavior is hard to predict at scale - a tracing mechanism is essential.
- A Decision Table is a production rule system without chaining.
