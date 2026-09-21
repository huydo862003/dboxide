---
_type: Concept
_label: "Lexer"
status: 'learning'
tags:
  - 'parsing'
depends_on:
  - fref("02-research/04-glossary/chomsky-hierarchy.td")
---

The first stage of Syntax-Directed Translation. A lexer (also called scanner or tokenizer) converts raw text into a stream of tokens, each with a type and content. `state idle` becomes `[{type: state-keyword, content: "state"}, {type: identifier, content: "idle"}]`.

Usually implemented with a Regex Table Lexer: a list of regex-to-token-type rules applied in order. Lexing is typically done with a finite-state machine, while syntactic analysis uses a push-down machine.
