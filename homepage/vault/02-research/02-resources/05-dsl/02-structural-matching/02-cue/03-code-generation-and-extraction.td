---
_type: Book
_label: "Code Generation and Extraction"
url: "https://cuelang.org/docs/concept/code-generation-and-extraction-use-case/"
tags:
  - "dsl"
  - "type system"
  - "structural matching"
---

CUE use cases:

- Extracting definitions from existing sources.
- Enriching definitions with constraints.
- Converting definitions to other formats.

## Extracting from Existing Sources

CUE can extract type definitions from Go code and Protobuf definitions, combine constraints from multiple sources, and identify inconsistencies.

## Enhancing Existing Standards

CUE allows annotating existing sources with CUE constraints inline. For Protobuf:

```proto
message Server {
  int32 port = 1 [(cue.val) = ">5000 & <10_000"];
}
```

For Go, field tags serve the same purpose.

## Converting to Other Standards

CUE supports conversion to OpenAPI and Go. Other targets are possible.

## Comparisons

**CEL (Common Expression Language)**: a standardized expression language focused on simplicity and speed. CUE's unification is pseudo-linear; comprehensions introduce polynomial complexity.

**Protoc-gen-validate (PGV)**: similar Protobuf field annotation for validation rules.

```proto
message Server {
  int32 port = 1 [(validate.rules).int32 = { gte: 5000, lte: 10000 }];
}
```

CUE's advantage over PGV is that constraints are CUE values - they participate in unification and can be simplified, reasoned about, and converted to other formats.
