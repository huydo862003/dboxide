---
_type: Concept
_label: "SAX"
status: 'learning'
tags:
  - 'parsing'
  - 'xml'
---

Simple API for XML. An event-based XML parser that fires events ("opening tag", "text", "closing tag") as it reads the document top to bottom, never loading the full tree into memory. You only handle one piece at a time.

Contrast with DOM parsing, which loads the entire document into a tree structure first.

Used as an analogy for call-stack-based parsers: both process one node at a time without holding the full structure in memory.
