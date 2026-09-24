---
_type: Concept
_label: "Carrier Syntax"
status: 'learning'
tags:
  - 'dsl'
depends_on:
  - fref("02-research/04-glossary/internal-dsl.td")
  - fref("02-research/04-glossary/external-dsl.td")
---

The syntax that a DSL is embedded within. The carrier syntax handles structural scaffolding (delimiters, quoting, nesting) while the DSL provides domain-specific vocabulary on top.

- For an internal DSL, the carrier syntax is the host language. Examples: Ruby, Java.
- For an external DSL, it is typically XML, JSON, or a custom syntax.
