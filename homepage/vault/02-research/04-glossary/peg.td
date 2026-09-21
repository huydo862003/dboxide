---
_type: Concept
_label: "PEG"
status: 'learning'
tags:
  - 'parsing'
  - 'language theory'
depends_on:
  - fref("02-research/04-glossary/chomsky-hierarchy.td")
---

Parsing Expression Grammar. A grammar formalism that can handle most context-free situations and some context-sensitive ones. PEG parsers do not require separate lexing and tend to be more usable than context-free grammars in many situations. Still relatively new, with fewer mature tools. Best known PEG parsers are Packrat parsers. ANTLR has incorporated many ideas from PEGs.
