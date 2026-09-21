---
_type: Concept
_label: "Parser Combinator"
status: 'learning'
tags:
  - 'parsing'
  - 'external dsl'
depends_on:
  - fref("02-research/04-glossary/recursive-descent-parser.td")
  - fref("02-research/04-glossary/bnf.td")
---

A parsing approach where each grammar rule becomes an object, and you compose these objects together to build a parser. Follows the same algorithm as a Recursive Descent Parser but represents the grammar explicitly in code rather than burying it in control flow. Preferred when you do not want to use a Parser Generator but still want the grammar to be readable.
