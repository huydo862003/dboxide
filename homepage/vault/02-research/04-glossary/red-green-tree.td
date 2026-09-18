---
_type: Concept
_label: "Red-Green Tree"
status: 'mastered'
tags:
  - 'syntax tree'
depends_on:
  - fref("02-research/04-glossary/green-node.td")
---

A tree architecture that separates immutable storage (Green layer) from navigation and position information (Red layer). `GreenNode`s store actual data and can be shared across threads. `SyntaxNode`s (Red layer) are lightweight cursors that add parent pointers and compute positions on-demand. This separation enables efficient tree edits and incremental parsing.
