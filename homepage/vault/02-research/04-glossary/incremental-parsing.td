---
_type: Concept
_label: "Incremental Parsing"
status: 'mastered'
tags:
  - 'incremental compilation'
depends_on:
  - fref("02-research/04-glossary/green-node.td")
  - fref("02-research/04-glossary/red-green-tree.td")
---

A technique that reparses only the portions of a file affected by edits rather than reparsing the entire file. Enabled by immutable tree structures (GreenNode) that can be efficiently patched by swapping subtree pointers. Often not worth the implementation complexity as modern parsers are fast enough for full reparse.
