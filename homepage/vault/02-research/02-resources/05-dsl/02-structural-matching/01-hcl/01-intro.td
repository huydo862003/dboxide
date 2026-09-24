---
_type: Book
_label: "Introduction to HCL"
url: "https://hcl.readthedocs.io/"
tags:
  - "dsl"
  - "structural matching"
  - "config language"
---

::: note
These are the important syntatic concepts of HCL:

- **Argument**: Key-value pair of form `key = value`.
- **Block**: A container with a label and body, creating a nesting level.
- **Body**: A collection of arguments and blocks inside a nesting level.
:::

HCL configuration is built from two main constructs:

- **arguments** (key-value pairs)
- **blocks** (labeled containers with a body)

```hcl
io_mode = "async"

service "http" "web_proxy" {
  listen_addr = "127.0.0.1:8080"

  process "main" {
    command = ["/usr/local/bin/awesome-app", "server"]
  }

  process "mgmt" {
    command = ["/usr/local/bin/awesome-app", "mgmt"]
  }
}
```

- `io_mode` is a top-level argument.
- `service` introduces a block.
  - `"http"` and `"web_proxy"` are **labels** on that block.
- Within a block body, further arguments and nested blocks are allowed.
- The keywords `io_mode`, `service`, `process` are all application-defined. HCL only provides the block structure syntax.

## Bodies

The **collection** of **arguments** and **blocks** at a particular nesting level is called a **body**.

- A **file** has a **root body** containing **top-level elements**.
- Each **block** has its **own body**.

::: note
The term "attribute" and "argument" are used interchangeably for top-level key-value pairs in a body.

- "Attribute" is also used for fields of object values inside expressions.
- "Argument" is preferred when referring specifically to body-level assignments.
:::

## Argument Expressions

Argument values can be literals or expressions with arithmetic, references, and function calls.

```hcl
listen_addr = env.LISTEN_ADDR
```

- **Built-in arithmetic and comparison operators are always available**.
- The calling application can optionally provide variables (like `env` above) and custom functions.

## JSON Alternative Syntax

The same structure can be expressed as JSON:

```json
{
  "io_mode": "async",
  "service": {
    "http": {
      "web_proxy": {
        "listen_addr": "127.0.0.1:8080",
        "process": {
          "main": {
            "command": ["/usr/local/bin/awesome-app", "server"]
          },
          "mgmt": {
            "command": ["/usr/local/bin/awesome-app", "mgmt"]
          }
        }
      }
    }
  }
}
```

- JSON syntax is ambiguous in places (e.g. whether a JSON object represents a nested block or an object expression). The application-provided schema resolves the ambiguity.
- JSON does not have an expression syntax, so HCL native expressions are embedded inside JSON strings when needed.
- The calling application chooses which syntaxes to support. JSON is optional.

::: tip
HCL is a **structured configuration language**, not a data serialization format. Unlike JSON, YAML, or TOML, HCL is always decoded against an application-defined schema. The schema is what gives meaning to the structure.
:::

> DBML today is a structured config language with a hardcoded adhoc schema, varying per builtin. The goal is to make the schema user-defined so the parser is reusable across formats.
