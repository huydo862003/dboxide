---
_type: Concept
_label: "Command-Query Separation"
status: 'learning'
tags:
  - 'api design'
depends_on:
  - fref("02-research/04-glossary/command-query-api.td")
  - fref("02-research/04-glossary/fluent-interface.td")
---

A principle that divides an object's methods into two categories: **queries** return a value without changing observable state, and **commands** change state without returning a value. This makes it easy to identify side-effect-free methods since you can call queries multiple times without worry.

Fluent interfaces (method chaining) typically break this principle because each method returns `this` to continue the chain. This is acceptable because fluent interfaces follow different design rules than command-query APIs.
