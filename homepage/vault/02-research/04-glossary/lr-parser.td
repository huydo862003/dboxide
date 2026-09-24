---
_type: Concept
_label: "LR Parser"
status: 'learning'
tags:
  - 'parsing'
depends_on:
  - fref("02-research/04-glossary/ll-parser.td")
---

A bottom-up parser. "L" means left-to-right scan, "R" means rightmost derivation. Reads tokens, shifts them onto a stack, and reduces when a grammar rule matches (shift-reduce parsing). More powerful than LL parsers but harder to understand and debug. Rarely needed for DSLs.

- Yacc is the classic example.
- Variants: LALR, GLR, SLR.
