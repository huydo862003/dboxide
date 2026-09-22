---
_label: "Structural Matching & Config Languages"
_icon: icon.sun
---

Resources on how to define and match structural patterns in syntax.

- **HCL Language Design** (HashiCorp).
  - Separates structural blocks from expressions.
    - The parser returns a generic tree of blocks and attributes without knowing what is valid.
    - Each application (Terraform, Packer, Nomad) defines its own schema specifying which block types exist and what fields they accept.
    - Validation happens separately by matching the parsed tree against the schema.
    > This is like the relaxed AST in DBML, except that DBML doesn't really allow defining schema flexibly.
  - This is Fowler's DSL pipeline: parse into a syntax tree, then translate into a semantic model, with a schema defining what is valid.
  - HCL has a bidirectional mapping to JSON. Any HCL file can be expressed as JSON and vice versa. The native syntax is for humans, JSON is for machines to generate config programmatically.
  - Link: [hcl.readthedocs.io](https://hcl.readthedocs.io/)
- **CUE: The Logic of CUE** by Marcel van Lohuizen.
  - Types and values unified into a single lattice, making type checking and validation the same operation.
  - The closest model to structural type checking where validation is not a separate pass but part of the type system itself.
  - Link: [cuelang.org](https://cuelang.org/docs/concept/the-logic-of-cue/)
