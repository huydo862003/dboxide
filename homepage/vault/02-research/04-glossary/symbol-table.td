---
_type: Concept
_label: "Symbol Table"
status: 'learning'
tags:
  - 'parsing'
  - 'dsl'
depends_on:
  - fref("02-research/04-glossary/syntax-tree.td")
---

A dictionary mapping identifier names to their objects, used during parsing to resolve cross-branch references. When an identifier is defined, it is stored in the symbol table. When it is referenced elsewhere in the script, the parser looks it up to wire the relationship.
