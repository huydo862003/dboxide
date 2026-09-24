---
_label: "HCL Language Design"
_icon: icon.sun
---

## Pipeline

```
raw bytes
    │
    ▼ hclparse.Parser (no schema)
*hcl.File { Body: opaque hcl.Body }
    │  - structure recognized (blocks, attributes, expressions)
    │  - no validity check; JSON ambiguity unresolved
    │
    ▼ Structural decoding (Body + schema)
blocks + attributes as hcl.Expression (unevaluated)
    │  - schema enforces which block types and fields are valid
    │  - JSON ambiguity resolved here by the schema
    │
    ▼ Expression evaluation (hcl.Expression + hcl.EvalContext)
cty.Value
    │  - variables and functions injected via EvalContext
    │  - only phase that requires the eval context
```

> The key insight: parsing is schema-free, so the same parser works for any HCL-based language. The schema is what gives meaning - applied only at decode time.

## Foreword

HCL is a **toolkit** for **creating structured configuration languages**, not a single language. It provides:

- Parser:
  - Understand constructs (blocks, attributes, expressions) that a calling application assembles into its own configuration language.
  - Returns a generic tree of blocks and attributes without knowing what is valid.
  - Includes an expression syntax for inline computation, and with application support, variables and functions for more dynamic configs.
    > I really want to support this too!
- User-defined schemas:
  - Each application (Terraform, Packer, Nomad) defines its own schema specifying which block types exist and what fields they accept.
  - Validation happens separately by matching the parsed tree against the schema.

> This is like the relaxed AST in DBML, except that DBML doesn't really allow defining schema flexibly.

Some other properties:

- Has both a native syntax (human-friendly, inspired by libucl and nginx config) and a JSON variant (machine-friendly). The two are bidirectionally mappable.
- Currently Go-only via its library API.
- Link: [hcl.readthedocs.io](https://hcl.readthedocs.io/)
