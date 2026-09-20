---
_type: Concept
_label: "Grammar"
status: 'learning'
tags:
  - 'parsing'
  - 'dsl'
depends_on:
  - fref("02-research/04-glossary/syntax-tree.td")
---

A set of production rules that describe how a text stream is turned into a syntax tree. Each rule has a term and a definition of how it breaks down, e.g. `additionStatement := number '+' number`.

A language can have multiple valid grammars. There is no single "the grammar" for a language. Each grammar defines a different tree structure for the same text.

A grammar defines syntax only, not semantics.
