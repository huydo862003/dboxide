---
_type: Concept
_label: "TokenSource"
status: 'mastered'
tags:
  - 'architecture'
---

A trait interface that defines how parsers read input tokens. Provides methods like `current()`, `lookahead_nth()`, and `bump()`. Abstracts over different token sources (source files, macro expansions, synthetic tokens) so the same parser can handle all inputs.
