---
_type: Concept
_label: "Gradual Evaluation"
status: 'learning'
tags:
  - 'dsl design'
---

A multi-pass evaluation strategy where expressions are re-evaluated as more values become available, using typed unknown placeholders for unresolved values.

- Type checking works across all passes because unknowns carry type information without a concrete value.
- Lets staged systems validate structure early and defer value resolution until later.
