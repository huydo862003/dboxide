---
_type: Concept
_label: "Textual Macro"
status: 'learning'
tags:
  - 'dsl'
  - 'metaprogramming'
depends_on:
  - fref("02-research/04-glossary/macro.td")
---

A macro that performs simple text substitution before the file is processed. Can take parameters: the C preprocessor's `sqr(x)` expands to `x * x`. Fallen out of favor due to awkward edge cases (no scope, no type awareness, surprising substitutions).
