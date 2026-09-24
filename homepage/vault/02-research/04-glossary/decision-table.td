---
_type: Concept
_label: "Decision Table"
status: 'learning'
tags:
  - 'computational model'
  - 'dsl'
depends_on:
  - fref("02-research/04-glossary/adaptive-model.td")
---

A table of conditions and consequences. Each column represents one rule: match the conditions in that column, apply its consequences.

- The imperative equivalent (nested if/else) forces an execution order that the table does not imply.
- The table can check for completeness (missing permutations) automatically.
- One of the few DSL forms where domain experts can directly edit the rules via a spreadsheet.
