---
_type: Concept
_label: "Tree-Agnostic Parser"
status: 'mastered'
tags:
  - 'parsing'
depends_on:
  - fref("02-research/04-glossary/token-source.td")
  - fref("02-research/04-glossary/tree-sink.td")
---

A parser that emits abstract parsing events without knowledge of the final tree structure. By working through trait interfaces (TokenSource/TreeSink), the parser remains decoupled from tree representation, enabling the same parsing logic to produce different output formats.
