---
_type: Concept
_label: "Lossless Syntax Tree"
status: 'mastered'
tags:
  - 'syntax tree'
---

A tree representation that preserves everything from the source code including whitespace, comments, and invalid tokens. Enables perfect reconstruction of the original source text by concatenating token text. Invalid input gets wrapped in `ERROR` nodes rather than being discarded.
