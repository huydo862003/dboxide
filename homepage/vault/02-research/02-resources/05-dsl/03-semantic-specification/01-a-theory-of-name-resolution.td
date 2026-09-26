---
_type: Book
_label: "A Theory of Name Resolution"
url: "https://link.springer.com/chapter/10.1007/978-3-662-46669-8_9"
author: "Neron et al."
date: "2015"
tags:
  - "name resolution"
  - "scope graphs"
  - "type system"
---

**Scope graphs** as a **language-independent** **formalism** for **name binding**.

- Language-independent: The formalism can express name binding for a wide range of languages.
- Formalism: A formal mathematical model.
- Name binding: Express declaration and references of entities via name.

PDF link: https://eelcovisser.org/publications/2015/NeronTVW15.pdf

## Context: Spoofax

Spoofax is a ${fref("02-research/04-glossary/language-workbench.td")}.

A **language workbench** is a tool for defining a DSL together with its editing environment.

This paper provides:

- **The part of the theoretical foundation behind Spoofax's name resolution engine**.
- **Scope graphs**, which were developed to give Spoofax a _declarative, language-independent_ model for **name binding** (that could be used to drive IDE features like go-to-definition and find-references).

## Abstract

- A theory that is:
  - **Language-independent**
  - For **name-binding and resolution**
  - Suitable for programming languages with various **scoping rules** including **lexical scoping** and **modules**.
- Modeling **name resolution** as a **2-step problem**:
  1. Construct a **scope graph** from **language-specific** ASTs using **language-specific** rules.
  2. References in the scope graph are resolved to corresponding declarations using a **language-independent** resolution process.
- Devise:
  - A **resolution calculus**: a concise, declarative, language-independent specification of name resolution.
  - A **resolution algorithm** that is sound and complete with respect to the calculus.
  - Language-independent definitions of:
    - **alpha-equivalence**
    - **rename refactoring**
    from the calculus.
- Shows the approach covers a range of name binding patterns found in existing languages.

## Introduction

**Motivation**

- Naming is pervasive in language design and implementation.
  - Names identify declarations (variables, functions, types, modules).
  - Names allow declarations to be referenced from elsewhere.
- **Name resolution** underlies most language operations:
  - Static checking.
  - Translation.
  - Mechanized semantics.
  - IDE services.

**Problem**

- Resolution cuts across the **local inductive structure** of programs.
  - Example: A name introduced by a `let` node may be referenced by an arbitrarily distant child node.
- Languages with **explicit namespaces** add further complexity.
  - Example: Resolving a Java qualified reference requires first resolving the class/package, then the member within that context.

_Fortunately_, despite this diversity, the basic concepts of resolution reappear in similar form across lexically-scoped languages.

**Current limitations**

- In practice, **resolution rules are described informally and ad hoc**. Even in formalized languages, resolution rules are encoded as part of static/dynamic judgments tailored to the specific language.

::: details What are static and dynamic judgments?
Two main kinds of rules in a language spec:

- **Static judgments** = type-checking rules. Example: `Γ ⊢ x : T   if (x : T) ∈ Γ`. The lookup `(x : T) ∈ Γ` is name resolution, but it is embedded inside the typing rule.
- **Dynamic judgments** = evaluation rules. Example: `⟨x, σ⟩ ⇓ σ(x)`. Looking up `x` in the store `σ` is again name resolution, embedded inside the evaluation rule.

"Tailored to the specific language" means the environment `Γ` or store `σ` is defined specifically for that language. There is no standalone, reusable account of what name lookup means: if you want to build an IDE, you have to re-derive the resolution logic from scratch by reading those rules.
:::

- Implications:
  - The same resolution rules are encoded multiple times for different tools (symbol table in a compiler, use-to-definition in an IDE, substitution in a mechanized proof).

    -> This causes duplication and risks inconsistencies.
  - Contrast with syntax:
    - Context-free grammars are a well-established declarative formalism that underpins a wide variety of tools.
    - Name resolution lacks an equivalent.

**Idea/Approach for Formalizing Resolution**

So the idea is to give **name resolution in lexically-scoped languages** a formalism similar to CFG.

The formalism includes:

- A **scope graph** represents the **naming structure** of a program, _abstracting away from AST details_.
  - Nodes:
    - Name references
    - Declarations
    - **Scopes** (minimal program regions that behave uniformly with respect to name resolution)
  - Edges: associate
    - References to scopes
    - Declarations to scopes
    - Scopes to parent scopes (lexical nesting).
  - **Named scopes** (like modules or namespaces) and **import edges** extend the model to explicit namespaces, supporting transitive and cyclic imports.
- A **resolution calculus** specifies how to construct a _path through the scope graph from a reference to a declaration_.
  - Hiding (shadowing) is modeled by an ordering on resolution paths.
  - Ambiguous references correspond to multiple paths from the same reference node.
  - Unresolved references correspond to the absence of paths.
- The formalism yields language-independent definitions of **alpha-equivalence**, **valid renaming**, and **static environments** mapping bound identifiers to their declaration sites.
- Any tool relying on use-to-definition information (IDE code completion, live variable analysis) should be specifiable using scope graphs.

Scope graph construction is language-dependent:
  - For any given language, construction is specified by a syntax-directed definition over the grammar.
  - The paper illustrates this for a small example language but does not provide a generic binding specification language.

    **Relationship to NaBL**. NaBL is a DSL integrated into Spoofax that provides high-level, non-algorithmic descriptions of name binding rules.
    - It serves phase 1: describing how to construct a scope graph from a language's AST.
    - Phase 2 (resolution) is handled by the language-independent scope graph machinery.

    NaBL works well in practice but is defined largely by example and lacks a formal semantics. Scope graphs can provide that semantics for NaBL specifications.

> Essentially, each language provides a procedure to return a language-independent scope graph for name resolution.

**Relationship to Related Work**

Two well-studied topics in name binding:

1. **Representing already-resolved programs** (e.g. de Bruijn indices, nominal logic) in a way that makes binding structure explicit and supports program manipulation modulo alpha-equivalence. These approaches only work after resolution is already done.
   - Scope graphs differ in three ways:
     - The representation is independent of the underlying language grammar and AST.
     - Scope graphs represent **unresolved** programs and serve as both the input to resolution and the container for results. This means they support **ill-formed programs** (ambiguous or undefined references), which are the normal case in IDEs and front-end tools.
     - They support **explicit namespaces** (modules, OO classes).
2. **Binding specification languages**: enriched grammar descriptions that simultaneously specify language syntax and binding structure.
   - This work is complementary to the scope graph approach (scope graphs do not specify syntax).

**Contributions**

- **Scope graph and resolution calculus**: a language-independent framework capturing references, declarations, scopes, and imports, with a declarative resolution path calculus.
- **Variants**: extensions to the core framework supporting more complex binding schemes.
- **Coverage**: the framework covers a wide range of binding patterns let bindings (various flavors), qualified names, and Java inheritance.
- **Scope graph construction**: syntax-directed construction for a simple example language.
- **Resolution algorithm**: deterministic, terminating algorithm based on binding environments, proven sound and complete with respect to the calculus.
- **Alpha-equivalence and renaming**: language-independent characterization of alpha-equivalence and valid renaming.
