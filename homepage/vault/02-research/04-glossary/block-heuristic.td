---
_type: Concept
_label: "Block Heuristic"
status: 'mastered'
tags:
  - 'incremental compilation'
---

An incremental parsing optimization that reparses only the smallest `{}` block containing an edit. This works because parsers can maintain structurally balanced braces even in broken code by inserting implicit closing braces and wrapping errors. Often not worth the complexity since full reparse is typically fast enough.
