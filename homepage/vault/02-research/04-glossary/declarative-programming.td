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

A programming style where you describe **what** the result should be, not **how** to compute it step by step. SQL, HTML, CSS, and Makefiles are declarative. The term is fuzzy: Fowler notes that "declarative" generally applies to approaches that move away from the imperative model. In DSL contexts, it usually means configuring an adaptive model by describing its structure rather than issuing commands. Most DSLs lean declarative because their purpose is to populate a semantic model, not to execute arbitrary logic.
