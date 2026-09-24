---
_type: Concept
_label: "Gradual Evaluation"
status: 'learning'
tags:
  - 'dsl design'
---

A multi-pass evaluation strategy where expressions are evaluated repeatedly as more input values become available, using typed unknown placeholders for values not yet resolved. Each pass produces a more complete result. Type checking can still be performed across all passes because unknowns carry type information even without a concrete value. Enables distributed or staged systems to validate configuration structure early and defer value resolution until later.
