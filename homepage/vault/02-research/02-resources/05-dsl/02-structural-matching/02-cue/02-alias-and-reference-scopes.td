---
_type: Book
_label: "Alias and Reference Scopes"
url: "https://cuelang.org/docs/concept/alias-and-reference-scopes/"
tags:
  - "dsl"
  - "type system"
  - "structural matching"
---

How CUE resolves field references within struct definitions, and how aliases fix common scoping mistakes.

## Problem #1

Goal: share `config.metadata` into `serviceA` and `serviceB` so that when `config` is instantiated with concrete values, both services inherit them.

Failed attempt: reference `metadata` by name inside each service struct.

```cue
#Config: {
  metadata: #Metadata

  serviceA: #BaseConfig & {
    metadata: metadata
  }

  serviceB: #BaseConfig & {
    metadata: metadata
  }
}

config: #Config & {
  metadata: {
    name:      "test"
    namespace: "dev"
  }
}
```

Expected result: `config` defines both `serviceA` and `serviceB` with the `metadata: { ... }`

Output:

```bash
cue export problem1.cue --out yaml
config.serviceA.metadata.name: incomplete value string:
    ./problem1.cue:2:13
    ./problem1.cue:7:12
config.serviceA.metadata.namespace: incomplete value string:
    ./problem1.cue:3:13
    ./problem1.cue:7:12
```

### Explanation

`metadata: metadata` is a reference to itself, not to the `metadata` field in the outer scope. It is effectively equivalent to `metadata: _` (top), adding no constraints.

```
config.serviceA.metadata.name: incomplete value string
config.serviceA.metadata.namespace: incomplete value string
```

## Problem #2

Goal: same as above.

Failed attempt: use an explicit path `#Config.metadata` to avoid the self-reference.

```cue
serviceA: #BaseConfig & {
  metadata: #Config.metadata
}
```

Same output.

### Explanation

`#Config.metadata` is resolved at the time `#Config` is defined, not when `config` is instantiated (kind of like macro pasting). At definition time, `#Config.metadata` is just `#Metadata`, the abstract constraint with `name: string` and `namespace: string`. The concrete values (`name: "test"`, `namespace: "dev"`) are supplied later via `config: #Config & { metadata: { ... } }`, but this reference cannot see them.

## Solution: Aliases

```cue
#Config: {
  M=metadata: #Metadata

  serviceA: #BaseConfig & {
    metadata: M
  }

  serviceB: #BaseConfig & {
    metadata: M
  }
}

config: #Config & {
  metadata: {
    name:      "test"
    namespace: "dev"
  }
}
```

Output:

```yaml
config:
  metadata:
    name: test
    namespace: dev
  serviceA:
    metadata:
      name: test
      namespace: dev
  serviceB:
    metadata:
      name: test
      namespace: dev
```

`M` is a relative reference inside `#Config`. It resolves at the point of use, not at declaration time. When `#Config` is unified with concrete values, `M` points to the constraint and the concrete values together.
