---
_type: Concept
_label: "Foreign Code"
status: 'learning'
tags:
  - 'external dsl'
  - 'dsl pattern'
---

Embedding a general-purpose language inside an external DSL to handle rare special cases without growing the DSL into a general-purpose language. The foreign code is not parsed by the DSL parser. It is slurped as a string by the lexer and stored in the semantic model for later execution. Requires Alternative Tokenization to distinguish the embedded code from the DSL syntax.
