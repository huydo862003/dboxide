---
_label: "Structural Matching & Config Languages"
_icon: icon.sun
---

Resources on how to define and match structural patterns in syntax.

- **HCL Language Design** (HashiCorp).
  - Separates structural blocks from expressions. Each application defines its own block types.
  - Relevant to how a structural DSL can define construct shapes without being tied to a specific language.
  - Link: [hcl.readthedocs.io](https://hcl.readthedocs.io/)
- **CUE: The Logic of CUE** by Marcel van Lohuizen.
  - Types and values unified into a single lattice, making type checking and validation the same operation.
  - The closest model to structural type checking where validation is not a separate pass but part of the type system itself.
  - Link: [cuelang.org](https://cuelang.org/docs/concept/the-logic-of-cue/)
