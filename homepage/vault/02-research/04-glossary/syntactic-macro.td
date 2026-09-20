---
_type: Concept
_label: "Syntactic Macro"
status: 'learning'
tags:
  - 'dsl'
  - 'metaprogramming'
depends_on:
  - fref("02-research/04-glossary/macro.td")
---

A macro that operates on syntactically valid expressions, transforming one kind of expression into another. Unlike textual macros, it is aware of the language's structure. Lisp is the canonical example; C++ templates are a more familiar one. Core to building internal DSLs in Lisp, but only available in languages that support them.
