---
_type: Concept
_label: "Salsa"
status: 'mastered'
tags:
  - 'incremental compilation'
depends_on:
  - fref("02-research/04-glossary/incremental-parsing.td")
---

An incremental computation framework that enables on-demand, cached evaluation of derived data. When inputs change, Salsa tracks dependencies and recomputes only what's affected. Used in rust-analyzer's `base-db` crate to make analysis incremental and efficient.
