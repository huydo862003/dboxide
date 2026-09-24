---
_type: Concept
_label: "Parser Generator"
status: 'learning'
tags:
  - 'parsing'
  - 'external dsl'
depends_on:
  - fref("02-research/04-glossary/bnf.td")
---

A tool that takes a BNF grammar and generates a parser from it. The grammar is the source of truth. Most mature approach for building parsers, but adds a code generation step to the build process. Examples:

- ANTLR
- Yacc
- Bison
