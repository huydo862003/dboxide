---
_type: Concept
_label: "Ahead-of-time Evaluation"
status: 'learning'
tags:
  - 'dsl design'
depends_on:
  - fref("02-research/04-glossary/gradual-evaluation.td")
---

An evaluation strategy where all expressions in a configuration are fully resolved in one subsystem before results are passed to other subsystems. Only the resulting constant values cross the subsystem boundary, not the original source. Simpler than gradual evaluation and ensures consistency across subsystems, but requires all input values to be known at the point of evaluation. Contrasted with gradual evaluation, where source files are passed between subsystems and re-evaluated as more values become available.
