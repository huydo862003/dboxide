---
_type: Concept
_label: "Dependency Network"
status: 'learning'
tags:
  - 'computational model'
  - 'dsl'
depends_on:
  - fref("02-research/04-glossary/adaptive-model.td")
---

A model that defines tasks and their prerequisites. The system figures out execution order and avoids redundant work. Build tools (Make, Ant, Rake) are the most familiar example. A good fit when you have computationally expensive tasks with dependencies between them.
