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

## Section 2: Scope Graphs and Resolution Paths

This section defines scope graphs and a resolution calculus over them. It motivates why ASTs are insufficient for name resolution and introduces the full formal theory.

### Motivation: Why Not Resolve Directly on the AST?

- In simple cases (e.g. lambda abstraction), the scope of a bound variable is the subtree dominated by the binding construct.
- But many binding patterns break this. Examples:
  - ML's sequential `let`: the set of AST locations where bindings are visible does not form a contiguous subtree.
  - Function parameters: the parameter list is in a subtree that does not dominate the use positions.
- Informally, we can understand these by a conceptual mapping from the AST to an underlying pattern of scopes, but this mapping is never made explicit in conventional language descriptions.

### Language-Independent Concept: Scope Graph

- A scope graph is obtained by a **language-specific mapping** from a program's AST.
- The mapping collapses all AST nodes that behave uniformly with respect to name resolution into a single **scope** node. A scope is a maximal region of the AST where the set of visible names does not change.

::: note Example: collapsing AST nodes into scopes
```
{
  let x = 1;
  let y = 2;
  x + y
}
```
- After both `let`s, all AST nodes (`x + y`, `BinaryExpr`, `Ident "x"`, `Ident "y"`) see the same visible names `{x, y}`. They collapse into **one scope node**.
- Between `let x` and `let y`, only `{x}` is visible. That is a **different scope node**.
:::

- Two assumptions about the mapping:
  - It can be built in a **single traversal** of the AST.
  - It is **purely syntactic**: no name resolution is needed to construct it.

### Theory Overview

The full theory is defined in these three callouts:

::: note Reference & declaration, scope graph formulation

Scope graph contains references, declarations and scopes as nodes.

**References and declarations**

- $x_i^\mathbf{D}:S$: declaration with name $x$ at position $i$ and optional associated named scope $S$.
  - The declaration appears in some scope (via $\mathcal{D}$), which is where the name is declared.
  - The optional $:S$ is the scope the declaration "opens up" (e.g. a module body). When another scope imports this declaration, resolution follows an import edge into $S$ to find its contents.
- $x_i^\mathbf{R}$: reference with name $x$ at position $i$. References do not carry associated scopes because they do not introduce new scopes.

**Scope graph**

- $\mathcal{G}$: scope graph
- $\mathcal{S}(\mathcal{G})$: scopes $S$ in $\mathcal{G}$
- $\mathcal{D}(S)$: declarations $x_i^\mathbf{D}:S'$ in $S$
- $\mathcal{R}(S)$: references $x_i^\mathbf{R}$ in $S$
- $\mathcal{I}(S)$: imports $x_i^\mathbf{R}$ in $S$
- $\mathcal{P}(S)$: parent scope of $S$

**Well-formedness properties**

- $\mathcal{P}(S)$ is a partial function.
  - A scope can have at most one parent.
- The parent relation is well-founded.
  - A scope parent chain eventually ends.
- Each $x_i^\mathbf{R}$ and $x_i^\mathbf{D}$ appears in exactly one scope $S$.
  - $x$ is the name and $i$ is the AST position, so $x_i$ identifies a specific occurrence.
  - Each occurrence belongs to exactly one scope: the graph is well-partitioned with no occurrence shared between scopes.
:::

::: note Resolution paths, well-formedness, and specificity ordering

**Resolution paths**

$$s := \mathbf{D}(x_i^\mathbf{D}) \mid \mathbf{I}(x_i^\mathbf{R}, x_j^\mathbf{D}\!:\!S) \mid \mathbf{P}$$

$s$ is a single **step** in a resolution path. Three kinds of steps:

- $\mathbf{D}(x_i^\mathbf{D})$: a **declaration step**, selects declaration node $x_i^\mathbf{D}$ from the current scope node. Always the final step.
- $\mathbf{I}(x_i^\mathbf{R}, x_j^\mathbf{D}\!:\!S)$: an **import step**, takes you from the current scope node into $S$ (the associated scope of declaration $x_j^\mathbf{D}$, reached by resolving import reference $x_i^\mathbf{R}$).
- $\mathbf{P}$: a **parent step**, takes you from the current scope node to its parent scope node.

A full path $p$ is a sequence of steps concatenated with $\cdot$.

$$p := [\,] \mid s \mid p \cdot p \quad \text{(inductively generated)}$$

$$[\,] \cdot p = p \cdot [\,] = p$$

$$(p_1 \cdot p_2) \cdot p_3 = p_1 \cdot (p_2 \cdot p_3)$$

**Well-formed paths**

$$WF(p) \Leftrightarrow p \in \mathbf{P}^* \cdot \mathbf{I}(\_,\_)^*$$

- A well-formed path is the prefix before the final $\mathbf{D}$ step (see rule $R$): zero or more parent steps, followed by zero or more import steps.
- Paths like $\mathbf{P} \cdot \mathbf{I} \cdot \mathbf{P}$ are disallowed: you cannot traverse a parent step after an import step.
  - This means you cannot break out of an imported scope into the parent.
  - Currently, this means you can only import the whole scope.

**Specificity ordering on paths**

$<$ means "more specific than" (takes priority when shadowing). Lower in the step hierarchy = more specific = wins.

$$\frac{}{\mathbf{D}(\_) < \mathbf{I}(\_,\_)} \quad (DI)$$

A name found directly in a scope shadows the same name found via import.

$$\frac{}{\mathbf{I}(\_,\_) < \mathbf{P}} \quad (IP)$$

A name found via import shadows the same name found in a parent scope.

$$\frac{}{\mathbf{D}(\_) < \mathbf{P}} \quad (DP)$$

A name found directly in a scope shadows the same name found in a parent scope. (Derivable from $DI$ and $IP$, stated explicitly.)

$$\frac{s_1 < s_2}{s_1 \cdot p_1 < s_2 \cdot p_2} \quad (Lex1)$$

If the first step of one path is more specific than the first step of another, the whole path wins regardless of the tails.

$$\frac{p_1 < p_2}{s \cdot p_1 < s \cdot p_2} \quad (Lex2)$$

If two paths share the same first step, break the tie by comparing their tails.

$Lex1$ and $Lex2$ together define a lexicographic order on paths: compare step by step left to right, first difference wins.
:::

::: note Resolution calculus

**Edges in scope graph**

$$\frac{\mathcal{P}(S_1) = S_2}{\mathbb{I} \vdash \mathbf{P} : S_1 \longrightarrow S_2} \quad (P)$$

$$\frac{y_i^\mathbf{R} \in \mathcal{I}(S_1) \setminus \mathbb{I} \quad \mathbb{I} \vdash p : y_i^\mathbf{R} \longmapsto y_j^\mathbf{D}\!:\!S_2}{\mathbb{I} \vdash \mathbf{I}(y_i^\mathbf{R}, y_j^\mathbf{D}\!:\!S_2) : S_1 \longrightarrow S_2} \quad (I)$$

**Transitive closure**

$$\frac{}{\mathbb{I} \vdash [\,] : A \twoheadrightarrow A} \quad (N)$$

$$\frac{\mathbb{I} \vdash s : A \longrightarrow B \quad \mathbb{I} \vdash p : B \twoheadrightarrow C}{\mathbb{I} \vdash s \cdot p : A \twoheadrightarrow C} \quad (T)$$

**Reachable declarations**

$$\frac{x_i^\mathbf{D} \in \mathcal{D}(S') \quad \mathbb{I} \vdash p : S \twoheadrightarrow S' \quad WF(p)}{\mathbb{I} \vdash p \cdot \mathbf{D}(x_i^\mathbf{D}) : S \hookrightarrow x_i^\mathbf{D}} \quad (R)$$

**Visible declarations**

$$\frac{\mathbb{I} \vdash p : S \hookrightarrow x_i^\mathbf{D} \quad \forall j, p'(\mathbb{I} \vdash p' : S \hookrightarrow x_j^\mathbf{D} \Rightarrow \neg(p' < p))}{\mathbb{I} \vdash p : S \longmapsto x_i^\mathbf{D}} \quad (V)$$

**Reference resolution**

$$\frac{x_i^\mathbf{R} \in \mathcal{R}(S) \quad \{x_i^\mathbf{R}\} \cup \mathbb{I} \vdash p : S \longmapsto x_j^\mathbf{D}}{\mathbb{I} \vdash p : x_i^\mathbf{R} \longmapsto x_j^\mathbf{D}} \quad (X)$$
:::
