---
_type: Concept
_label: "HIR"
status: 'mastered'
tags:
  - 'architecture'
depends_on:
  - fref("02-research/04-glossary/ast.td")
---

High-level Intermediate Representation. A fully resolved, semantic view of the code that abstracts away syntactic details. In rust-analyzer, the `hir` crate provides a static API for querying resolved types, names, and semantic relationships, built on top of the syntax tree and incremental analysis passes.
