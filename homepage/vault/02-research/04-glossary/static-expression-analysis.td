---
_type: Concept
_label: "Static Expression Analysis"
status: 'learning'
tags:
  - 'dsl design'
depends_on:
  - fref("02-research/04-glossary/structural-decoding.td")
---

Inspection of an expression's syntax tree structure without evaluating it. Extracts information such as variable references, traversal paths, function call names, or list/map structure directly from the AST. Used for dependency graph construction, name resolution, reference validation, and cross-reference analysis - all before any values are computed. Complements structural decoding: structure is decoded without evaluating expressions, then static analysis extracts dependency information from those unevaluated expressions.
