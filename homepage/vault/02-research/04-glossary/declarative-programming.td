---
_type: Concept
_label: "Declarative Programming"
status: 'learning'
tags:
  - 'dsl design'
depends_on:
  - fref("02-research/04-glossary/computational-model.td")
  - fref("02-research/04-glossary/adaptive-model.td")
---

A style where you describe what the result should be, not how to compute it. SQL, HTML, CSS, and Makefiles are declarative.

- The term is fuzzy. In DSL contexts, it usually means configuring an adaptive model by describing its structure rather than issuing commands.
- Most DSLs lean declarative because their purpose is to populate a semantic model, not execute arbitrary logic.
