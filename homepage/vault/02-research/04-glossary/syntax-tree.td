---
_type: Concept
_label: "Syntax Tree"
status: 'learning'
tags:
  - 'parsing'
  - 'dsl'
depends_on:
  - fref("02-research/04-glossary/grammar.td")
---

A hierarchical representation of a parsed script. Nodes are constructs, children are their parts.

Often not a real data structure. It frequently lives implicitly on the call stack, forming and pruning as the parser walks the input. You only ever see the branch currently being processed (similar to SAX parsing).

In an internal DSL, the implicit tree is formed by:

- **Nested Function**: nesting via arguments.
- **Method Chaining**: nesting via chained calls.
- **Function Sequence + Context Variables**: no nesting; a shared mutable context simulates the hierarchy.

Distinct from the semantic model.

- The syntax tree captures syntactic structure.
- The semantic model captures meaning.
