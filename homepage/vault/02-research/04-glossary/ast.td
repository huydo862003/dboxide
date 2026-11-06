---
_type: Concept
_label: "AST"
status: 'mastered'
tags:
  - 'syntax tree'
depends_on:
  - fref("./lossless-syntax-tree.td")
---

Abstract Syntax Tree. A typed, high-level representation of source code structure that provides ergonomic access to syntactic constructs. In rust-analyzer, AST nodes like `FnDef` and `ParamList` are auto-generated wrappers around the lower-level `SyntaxNode`, offering convenient methods for traversing specific language constructs.
