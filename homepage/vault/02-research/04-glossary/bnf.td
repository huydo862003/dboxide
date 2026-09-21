---
_type: Concept
_label: "BNF"
status: 'learning'
tags:
  - 'parsing'
  - 'language theory'
depends_on:
  - fref("02-research/04-glossary/grammar.td")
---

Backus-Naur Form. A notation for writing grammar production rules. Each rule has a name and a definition: `eventBlock : 'events' eventDec* 'end';`. Variants include EBNF (Extended BNF) which adds `*`, `+`, `?` operators. Parser Generators use BNF or a close variant as their input language.
