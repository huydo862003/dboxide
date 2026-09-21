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

The syntax that a DSL is embedded within. For an internal DSL, the carrier syntax is the host language (Ruby, Java). For an external DSL, it can be XML, JSON, or a custom syntax. The carrier syntax handles the structural scaffolding (delimiters, quoting, nesting) while the DSL provides the domain-specific vocabulary on top.
