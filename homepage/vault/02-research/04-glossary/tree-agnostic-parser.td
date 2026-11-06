---
_type: Concept
_label: "Tree-Agnostic Parser"
status: 'mastered'
tags:
  - 'parsing'
depends_on:
  - fref("./token-source.td")
  - fref("./tree-sink.td")
---

A parser that emits abstract parsing events without knowledge of the final tree structure. By working through trait interfaces (TokenSource/TreeSink), the parser remains decoupled from tree representation, enabling the same parsing logic to produce different output formats.
