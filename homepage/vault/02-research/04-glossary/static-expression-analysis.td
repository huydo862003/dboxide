---
_type: Concept
_label: "Static Expression Analysis"
status: 'learning'
tags:
  - 'dsl design'
depends_on:
  - fref("02-research/04-glossary/structural-decoding.td")
---

Inspection of an expression's syntax tree structure without evaluating it. Extracts information directly from the AST:

- Variable references
- Traversal paths
- Function call names
- List/map structure

Used for dependency graph construction, name resolution, reference validation, and cross-reference analysis, all before any values are computed. Complements structural decoding: after structure is decoded, static analysis extracts dependency information from unevaluated expressions.
