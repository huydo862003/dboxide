---
_type: Concept
_label: "Trivia"
status: 'mastered'
tags:
  - 'syntax tree'
---

Non-semantic tokens like whitespace, comments, and formatting characters. Different parsers handle trivia differently: explicit nodes (trivia as sibling nodes), attached trivia (stored in leading/trailing properties of tokens), or linked lists (separate doubly-linked list alongside the tree). rust-analyzer uses explicit nodes for uniformity and losslessness.
