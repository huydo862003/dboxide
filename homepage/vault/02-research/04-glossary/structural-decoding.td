---
_type: Concept
_label: "Structural Decoding"
status: 'learning'
tags:
  - 'dsl design'
---

A parsing phase that recognizes structural constructs (blocks, nesting, attribute names) without evaluating the values of those attributes. The result is a tree of structural elements where attribute values remain as unevaluated expression handles. Expression evaluation is a separate, subsequent phase that requires an explicit context (available variables, functions). This separation lets different parts of the input be evaluated with different contexts, and enables static analysis of expression dependencies before any values are computed.
