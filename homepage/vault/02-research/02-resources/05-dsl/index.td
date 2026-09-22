---
_label: "DSL"
_icon: icon.sun
---

Resources about DSL design philosophy relevant to dboxide as a whole.

::: tip
The end goal is a DSL that can:

- Match a wide range of delimiter-based syntax (`()`, `{}`, `[]`) with expressions.
- Define name resolution rules.
- Define type checking rules.
- Describe corresponding data formats.
- Use an alternative computational model (production rules, constraints) for validating the data format.
:::

::: note

Key design takeaway from HCL:

- **Structural constructs** (blocks, delimiters, nesting) are generic and language-agnostic. The parser understands structure without knowing what is valid.
- **Expressions** provide dynamic computation within structures (interpolation, variables, functions).
- **Schemas** are user-defined, not hardcoded. The user specifies which block types exist, what fields they accept, and what constraints apply. Validation is a separate step that matches the parsed tree against the schema.

This means the parser is reusable across different config formats. Only the schema changes per application.
:::

[[directory-index]]
