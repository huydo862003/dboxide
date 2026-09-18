---
_type: Concept
_label: "Panic Mode"
status: 'mastered'
tags:
  - 'parsing'
---

An error recovery technique where the parser skips tokens until it finds a synchronization point (like `}` or `;`) after encountering a syntax error. Allows the parser to continue and find additional errors rather than stopping at the first one.
