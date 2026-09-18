---
_type: Concept
_label: "Resilient Parsing"
status: 'mastered'
tags:
  - 'parsing'
depends_on:
  - fref("02-research/04-glossary/panic-mode.td")
---

A parsing approach that always produces a tree even from syntactically invalid code. Employs error recovery techniques (panic mode, implicit token insertion, early termination) to maintain tree structure. Invalid sections get wrapped in `ERROR` nodes. Enables IDE features like completion to work even in broken code.
