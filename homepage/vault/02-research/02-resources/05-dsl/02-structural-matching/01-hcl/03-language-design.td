---
_type: Book
_label: "HCL Configuration Language Design"
url: "https://hcl.readthedocs.io/en/latest/language_design.html"
tags:
  - "dsl"
  - "structural matching"
  - "config language"
---

Guidelines for designing an HCL-based language that feels consistent with other HCL-based languages (Terraform, Nomad, etc.) and makes good use of HCL's building blocks. These are conventions, not requirements.

## Naming

- Use `snake_case` for all identifiers: arguments, block types, variables, functions.
- Block type names are singular (each block defines one object). Use plural only for pure namespacing containers that hold nested blocks and no arguments of their own.
- Argument names are singular unless the argument expects a collection, in which case use plural: `name = "foo"` but `subnet_ids = ["abc", "123"]`.
- Function names typically run words together without underscores, following C stdlib conventions (`strlen`, `substr`). Longer names may use underscores for readability.

## Blocks vs Object Values

Blocks and object constructor expressions look similar in native syntax but differ in important ways:

```hcl
# block
service "web" {
  listen_addr = "0.0.0.0:8080"
}

# argument with object constructor
tags = {
  env  = "prod"
  team = "platform"
}
```

- Use **blocks** for rigid domain objects with a known schema defined by the application. Blocks can contain nested blocks.
- Use **object constructors** for free-form data where the keys are chosen by the user and the application treats the result as a map.

```hcl
resource "aws_instance" "example" {
  ami           = "ami-abc123"  # argument: fixed schema
  instance_type = "t2.micro"

  tags = {                      # object constructor: user-defined keys
    Name = "example instance"
  }

  ebs_block_device {            # nested block: separate domain object with rigid schema
    device_name = "hda1"
    volume_size = 8
  }
}
```

- Special case: block syntax with free-form keys is acceptable when each key declares a distinct top-level object. Terraform's `locals` block uses this - each argument name is a new local variable:

```hcl
locals {
  instance_type = "t2.micro"           # declares local.instance_type
  instance_id   = aws_instance.x.id   # declares local.instance_id
}
```

## Block Results as Variables

When top-level blocks also declare variables available in expressions (interdependent blocks), the convention is:

- Expose a variable named after the block type, typed as an object or map reflecting the label structure.
- For a block `service "http" "web_proxy" { ... }`, the variable path is `service.http.web_proxy`.
- Nested blocks with unordered instances are best exposed as a map (keyed by label): `service.http.web_proxy.process["main"].command`.
- Nested blocks with significant ordering are better exposed as a list, enabling splat operators.

## Standard Functions

Applications should offer a consistent set of functions from the `cty` stdlib where applicable, so users of multiple HCL-based languages encounter compatible behavior:

`abs`, `coalesce`, `compact`, `concat`, `format`, `hasindex`, `int`, `jsondecode`, `jsonencode`, `length`, `lower`, `max`, `min`, `strlen`, `substr`, `upper`, and set operations: `sethas`, `setintersection`, `setsubtract`, `setsymdiff`, `setunion`.

- Not all functions make sense for every application. A format with no set types has no reason to offer set functions.
- Some languages offer no functions at all if their purpose is purely value assignment.

## File Extensions

- Define a file extension specific to your language (e.g. `.tf` for Terraform, `.hcldec` for hcldec specs) to enable editor integrations.
- Using `.hcl` is fine for simple languages where only basic syntax highlighting is needed.
- Editor integrations for a specific language should not match `.hcl` generically, as this causes confusing results when users write config for other applications.
