---
_type: Concept
_label: "TreeSink"
status: 'mastered'
tags:
  - 'architecture'
---

A trait interface that defines how parsers write output. Receives events like `start_node()`, `token()`, `finish_node()`, and `error()`. The implementation decides how to build the actual tree structure. This abstraction enables different tree formats from the same parser.
