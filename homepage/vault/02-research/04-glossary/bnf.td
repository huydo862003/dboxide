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

Backus-Naur Form is a notation for writing grammar production rules. Each rule has a name and a definition:

```
eventBlock : 'events' eventDec* 'end';
```

- EBNF (Extended BNF) adds `*`, `+`, `?` operators.
- Parser generators use BNF or a close variant as their input language.
