---
_type: Concept
_label: "Event-Based Parser"
status: 'mastered'
tags:
  - 'parsing'
depends_on:
  - fref("./token-source.td")
  - fref("./tree-sink.td")
---

A parser architecture where the parser emits abstract events (like `start_node`, `token`, `finish_node`) through trait interfaces (`TokenSource` for input, `TreeSink` for output) rather than directly constructing tree structures. This decouples the parsing algorithm from tree representation, enabling the same parser to work with different input sources and output formats.
