---
_type: Concept
_label: "Chomsky Hierarchy"
status: 'learning'
tags:
  - 'parsing'
  - 'language theory'
depends_on:
  - fref("02-research/04-glossary/grammar.td")
---

A classification of grammars by their expressive power. Three levels matter for DSLs:

- **Regular**: processable with a finite-state machine. Cannot handle nesting (parentheses, nested blocks). Regex is a regular grammar.
- **Context-free**: adds a stack, allowing nesting. Most programming languages and DSLs are context-free.
- **Context-sensitive**: meaning depends on surrounding context (e.g. variable must be declared before use). We cannot generate parsers for these, which is why we use symbol tables.

Each level is a strict superset of the one before it.
