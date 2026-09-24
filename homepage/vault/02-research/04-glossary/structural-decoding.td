---
_type: Concept
_label: "Structural Decoding"
status: 'learning'
tags:
  - 'dsl design'
---

A parsing phase that recognizes blocks, nesting, and attribute names without evaluating attribute values. The result is a tree where attribute values remain as unevaluated expression handles.

- Expression evaluation is a separate phase requiring an explicit context (variables, functions).
- Different parts of the input can be evaluated with different contexts.
- Static analysis of expression dependencies can run before any values are computed.
