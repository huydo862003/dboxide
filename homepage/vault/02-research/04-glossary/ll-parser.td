---
_type: Concept
_label: "LL Parser"
status: 'learning'
tags:
  - 'parsing'
depends_on:
  - fref("02-research/04-glossary/recursive-descent-parser.td")
---

A top-down parser. "L" means left-to-right scan, second "L" means leftmost derivation. Starts from the highest grammar rule and works down. Recursive Descent Parsers, Parser Combinators, and ANTLR are all LL-style. Cannot handle left recursion (`expr: expr '+' expr`) without rewriting the grammar. Almost always sufficient for DSLs.
