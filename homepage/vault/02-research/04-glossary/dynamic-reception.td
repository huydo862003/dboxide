---
_type: Concept
_label: "Dynamic Reception"
status: 'learning'
tags:
  - 'internal dsl'
  - 'dsl pattern'
depends_on:
  - fref("02-research/04-glossary/internal-dsl.td")
---

Fowler's term for intercepting undefined method calls at runtime to implement dynamic behavior. The pattern moves information from method arguments into method names: `people.find_by_firstname("martin")` instead of `people.find("firstname", "martin")`.

Known by different names in different languages:

- Ruby: `method_missing`
- Smalltalk: `doesNotUnderstand`
- Python: `__getattr__`

Best suited for simple, repetitive patterns like building queries by property name. Gets unwieldy for complex structures.
