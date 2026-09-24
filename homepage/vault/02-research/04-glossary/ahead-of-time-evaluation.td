---
_type: Concept
_label: "Ahead-of-time Evaluation"
status: 'learning'
tags:
  - 'dsl design'
depends_on:
  - fref("02-research/04-glossary/gradual-evaluation.td")
---

An evaluation strategy where all expressions are fully resolved before results cross a subsystem boundary - only constant values pass through, not the original source.

- Simpler than gradual evaluation and ensures consistency across subsystems.
- Requires all input values to be known at the point of evaluation.
