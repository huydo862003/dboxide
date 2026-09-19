---
_type: Concept
_label: "Fluent Interface"
status: 'learning'
tags:
  - 'dsl classification'
depends_on:
  - fref("02-research/04-glossary/internal-dsl.td")
  - fref("02-research/04-glossary/command-query-api.td")
---

A style of API design where method calls chain together to read like a declarative language. Fluent interfaces are the mechanism behind internal DSLs: instead of calling methods to get things done (command-query API), you compose declarations that read like a language. The term was coined by Martin Fowler and Eric Evans.
