---
_type: Book
_label: "The Logic of CUE"
url: "https://cuelang.org/docs/concept/the-logic-of-cue/"
tags:
  - "dsl"
  - "type system"
  - "structural matching"
---

## Types are Values

Two core properties distinguish CUE from typical configuration or programming languages:

- **Types are values**:
  - There is no distinction between a type constraint and a concrete value.
  -Both live in the same space.
- **Values are ordered into a lattice**:
  - Every pair of values (types) has a meet (greatest lower bound) and a join (least upper bound).
  - More specific values sit lower in the lattice.

### Implications

These two properties cause many concepts that are separate in other languages to fold together in CUE:

- Schema, validation, policy, and data are all the same operation: constraining a value in the lattice.
- A type is just a value that is less specific than a concrete value. `string` is above `"hello"` in the lattice.
- Unifying two values means computing their meet. Incompatible constraints produce bottom (`_|_`), which is a type error.

::: note
**Top and bottom:**

- `_` (top) matches anything. The most general value.
- `_|_` (bottom) is the error value. Produced when two constraints are incompatible. Nothing satisfies it.
:::

Because everything is ordered, CUE gains order independence: combining constraints in any order produces the same result. This simplifies reasoning for both humans and tools.

### Unique Features

Capabilities enabled by this model:

- A single language for data, schema, validation, and policy. No separate validation pass needed.
- Meta-reasoning: determining whether a new schema version is backwards compatible is just checking lattice ordering.
- Automated rewriting: `cue trim` removes redundant values that are already implied by constraints higher in the lattice.
- Multi-source constraint pipelines: merge constraints from multiple sources without conflict, retaining documentation across normalization.
- The lattice also forces **formal rigor**: concepts like "optional", "default value", and "null" must be defined precisely as positions in the lattice. There is no wiggle room for ambiguity. Every value either fits or it does not.

> Contrasted with HCL: HCL applies schema as a separate decode step after parsing. CUE has no separate validation step. Constraints are the values themselves.

### The Value Lattice

Every value in CUE (types, concrete values, entire configs, schemas) is ordered in a single lattice hierarchy.

The ordering relation is **subsumption**: A subsumes B if B is a more specific instance of A.

- `string` subsumes `"hello"`: `"hello"` is an instance of `string`.
- A config with fewer constraints subsumes one with more: the more constrained config is an instance of the less constrained one.

::: note
This subsumption ordering gives CUE a critical property: for any two values, their meet (the most specific value that satisfies both) is always unique and well-defined. Any two configurations can be merged unambiguously, regardless of order.
:::

## CUE's Hierarchy

A walkthrough of how CUE's value lattice applies to each built-in kind: booleans, numbers, strings, structs, and lists.

### Booleans

- `true` and `false` are both instances of `bool`.
- `bool` itself is a value, not just a type label.
- Meet (`&`): `bool & true` yields `true`. `true & false` yields bottom (error: a value cannot be both).
- Join (`|`): `true | false` yields `bool`.
- Bottom is an instance of every value and type. It represents an error.

```mermaid
graph TD
  bool --> true
  bool --> false
  true --> bottom["bottom (error)"]
  false --> bottom
```

### Numbers

- `number` is the top-level class. `int` and `float` are instances of `number`.
- Constraints like `>=0.5` and `<10` are values that sit between traditional types and concrete values. They define the set of all numbers A for which `A op B` is true.
  - Any binary operator that returns boolean (except `==`) can be used as a constraint by omitting the left operand: `<10` means all numbers less than 10.
  - Implicit conversion between number types applies in comparisons: `<10` covers both integers and floats less than 10.

```mermaid
graph TD
  number --> int
  number --> lt10["&lt;10"]
  number --> gte05[">=0.5"]
  int --> 0
  int --> 1
  lt10 --> 0
  lt10 --> 1
  lt10 --> 1.1
  gte05 --> 1
  gte05 --> 1.1
  gte05 --> 20.0
  number --> 20.0
  0 --> bottom["bottom (error)"]
  1 --> bottom
  1.1 --> bottom
  20.0 --> bottom
```

### CUE Types

The full type hierarchy from top to bottom:

```mermaid
graph TD
  top["top (_)"] --> number
  top --> bool
  top --> string
  top --> bytes
  top --> null
  top --> struct
  top --> list
  number --> int
  bool --> bottom["bottom (_|_)"]
  string --> bottom
  bytes --> bottom
  null --> bottom
  struct --> bottom
  list --> bottom
  int --> bottom
```

The `|` operator computes the least upper bound (join). It is a single operation used uniformly for:
  - Sum types: `int | string`
  - Enums: `1 | 2 | 3`
  - Defaults: `*1 | int` (the `*` marks the default value)
  - Null coalescing: `*pet.species | "cat"` evaluates to `pet.species`, or `"cat"` if it is null.

::: tip
These are not different uses of `|` via operator overloading. They are all the same lattice join operation applied to different positions in the hierarchy.
:::

### Structs

::: note
**Subsumption rule**: A struct B is an instance of struct A if B has all of A's fields with constraints at least as strict.
:::

Adding fields or tightening constraints moves a struct lower in the lattice (more specific).

```
municipality              { name: string, population: int }
  big city                { name: string, population: >1M }
    London                { name: "London", population: 8M }
```

Merging two structs means computing their meet: the result has all fields from both, with each field's constraint tightened to the meet of both constraints.

```mermaid
graph TD
  top["top"] --> a_int["{a: int}"]
  top --> b_int["{b: int}"]
  a_int --> a1["{a: 1}"]
  a_int --> ab_int["{a: int, b: int}"]
  b_int --> ab_int
  b_int --> b1["{b: 1}"]
  a1 --> ab1["{a: 1, b: 1}"]
  ab_int --> ab1
  b1 --> ab1
  ab1 --> bottom["bottom"]
```

- `{a: int} & {b: int}` produces `{a: int, b: int}`: both field constraints are kept.
- `{a: int} & {a: 1}` produces `{a: 1}`: the more specific constraint wins.
- Conflicting required fields produce bottom (error).

**Backwards compatibility analogy**: a new API version must subsume the old one. The old version must be an instance of the new one. A new version may not forbid what the old version allowed.

**Optional fields** (`a?`): required is more specific than optional. An instance may make an optional field required, but not remove it.

- Conflicting optional fields (`a?: 0` and `a?: 1`) produce `a?: bottom`, which means the field is disallowed entirely. The struct itself is still valid.
- Conflicting required fields (`a: 0` and `a: 1`) produce a faulty struct (bottom). A required field cannot be omitted.

The asymmetry follows from logic: from `P -> bottom` (contradiction), we can infer `not P`. If an optional field reaches bottom, we infer "not that field" and drop it. If a required field reaches bottom, it cannot be dropped, so the whole struct becomes faulty.

```mermaid
graph TD
  opt_int["a?: int"] --> opt_lt10["a?: int & &lt;10"]
  opt_int --> req_int["a: int"]
  opt_lt10 --> req_lt10["a: int & &lt;10"]
  req_int --> req_lt10
```

```mermaid
graph TD
  opt0["a?: 0"] --> opt_bottom["a?: bottom (field disallowed)"]
  opt1["a?: 1"] --> opt_bottom
  req0["a: 0"] --> bottom["bottom (faulty struct)"]
  req1["a: 1"] --> bottom
  opt_bottom --> bottom
```

::: details CUE's Theoretical Foundation

**Graph unification** of **typed feature structures** is the theoretical foundation of CUE.

:::
