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

A system where you write individual rules, each with a condition and an action. Unlike if/else, you do not specify evaluation order. The engine decides which rules to fire based on the current state of the data.

**Chaining**: firing one rule can change data that triggers other rules. This lets you write rules independently without thinking about interactions, but makes behavior implicit and hard to predict at scale. A tracing mechanism is essential.

A Decision Table is a Production Rule System without chaining, where each column is one independent rule.
