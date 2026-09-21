---
_type: Book
_label: "Choosing between Internal and External DSLs"
author: "Martin Fowler"
date: "2010"
description: "Tradeoffs between internal and external DSLs"
isbn: "978-0321712943"
url: "https://martinfowler.com/books/dsl.html"
tags:
  - "dsl"
  - "language design"
---

::: tip
- No general advantage either way.
- Start with an internal DSL if you want fast iteration and host language access.
- Move to an external DSL if you need syntactic freedom or domain expert readability.
- A semantic model makes switching cheap.
:::

## Learning Curve

- Internal DSLs seem easier because you already know the host language. But fluent interface tricks are unfamiliar territory, and you learn them gradually as you build.
- External DSLs require learning Syntax-Directed Translation upfront, which is a steeper initial investment. But once learned, writing a grammar and translation code is comparable in effort to building an Expression Builder layer.
- Internal DSLs also rely on obscure host language tricks that are hard to discover.

## Cost of Building

- For an internal DSL, the cost is building the Expression Builder layer over the model.
- For an external DSL, the cost is building the parser. With Syntax-Directed Translation, this is comparable to the Expression Builder cost.
- Both are incremental costs over the model itself, which you need regardless.

## Programmer Familiarity

- With an internal DSL, programmers use their existing IDE, debugger, and refactoring tools.
- With an external DSL, you usually fall back to a plain text editor with no autocompletion or type checking. A Class Symbol Table can recover some IDE support for internal DSLs.

## Communication with Domain Experts

- Internal DSLs carry host language syntax noise (parentheses, brackets, semicolons). This may not bother programmers but can be a barrier for domain experts.
- External DSLs have full syntactic freedom, making them easier to read for non-programmers.
- If you are unsure, start with an internal DSL and switch to external later. The semantic model stays the same.

## Mixing in the Host Language

- An internal DSL is just a convention over the host language. You can freely mix DSL code with regular imperative code. This is a strength when the DSL needs general-purpose features like arithmetic, loops, or abstractions.
- Build tools illustrate this well: Make and Ant (external DSLs) slide into generality trying to express complex build logic. Rake (Ruby internal DSL) freely mixes the dependency network with Ruby code.
- External DSLs can embed host code as Foreign Code, but the mixing is awkward. Tools do not know what you are doing, and you cannot easily share symbols between the two environments.

## Strong Expressiveness Boundary

- The thin boundary of an internal DSL is not always a strength. If domain experts are writing the DSL, mixing in host code raises a communication barrier.
- An external DSL's limited expressiveness is a feature: it restricts what can go wrong, serves as a barrier to bugs, and limits the surface area you need to test.

## Runtime Configuration

- External DSLs can be parsed at runtime without recompilation. This was a major driver behind XML config files.
- If you use an interpreted host language, internal DSLs get this for free.
- If your host language is compiled, you can use an interpreted language for the internal DSL (e.g. JRuby on the JVM), but this attenuates some of the internal DSL benefits (familiarity, tooling).

## Sliding into Generality

- A successful DSL tends to grow. Users ask for more features, and feature by feature (conditionals, loops, variables) it slides into a general-purpose language.
- This happens with both internal and external DSLs. Internal DSLs are less at risk because the host language already provides general-purpose features.
- Guard against it. If the DSL needs complex logic, introduce a second smaller DSL or use Foreign Code rather than expanding the first one.

## Composing DSLs

- Internal DSLs compose easily because they are just code in the host language.
- External DSLs are harder to compose. Most Parser Generators cannot combine grammars from different languages. You end up using Foreign Code, which is clunky.

