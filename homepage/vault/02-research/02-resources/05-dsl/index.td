---
_label: "General DSL Philosophy"
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

## DSL Design & Implementation

- **Domain-Specific Languages** by Martin Fowler (Addison-Wesley, 2010). The definitive catalog of DSL patterns: internal vs external, semantic models, code generation.
- **The Racket Manifesto** by Felleisen et al. (SNAPL 2015).
  - Articulates language-oriented programming and self-hosting DSL design.
  - Link: [PDF](https://www2.ccs.neu.edu/racket/pubs/manifesto.pdf)
- **The Evolution of Lua** by Ierusalimschy et al. (HOPL III, 2007).
  - How a config language evolved into a turing-complete embeddable language.
  - Link: [PDF](https://www.cs.tufts.edu/comp/250RTS/archive/roberto-ierusalimschy/lua-hopl-iii.pdf)
- **Crafting Interpreters** by Bob Nystrom.
  - Two complete language implementations: tree-walking interpreter and bytecode VM.
  - Covers parsing, scoping, closures, and value representation.
  - Link: [craftinginterpreters.com](https://craftinginterpreters.com/)

## Structural Matching & Config Languages

- **HCL Language Design** (HashiCorp).
  - Separates structural blocks from expressions. Each application defines its own block types.
  - Relevant to how a structural DSL can define construct shapes without being tied to a specific language.
  - Link: [hcl.readthedocs.io](https://hcl.readthedocs.io/)
- **CUE: The Logic of CUE** by Marcel van Lohuizen.
  - Types and values unified into a single lattice, making type checking and validation the same operation.
  - The closest model to structural type checking where validation is not a separate pass but part of the type system itself.
  - Link: [cuelang.org](https://cuelang.org/docs/concept/the-logic-of-cue/)

## Name Resolution & Type Checking

- **A Theory of Name Resolution** by Neron et al. (ESOP 2015).
  - Scope graphs as a language-independent formalism for name binding.
  - Link: [Spoofax bibliography](https://www.spoofax.dev/)
- **Scopes as Types** by van Antwerpen et al. (PACMPL 2018).
  - Connects scope graphs to type systems, showing that scoping and typing can be unified.
- **Statix Constraint Language** by van Antwerpen et al. (PEPM 2016).
  - Declaratively specifying name binding and typing rules as part of the language definition.
  - The closest existing system to structural matching with integrated name resolution.
  - Link: [spoofax.dev](https://www.spoofax.dev/)

## Alternative Computational Models for Validation

- **Datalog** - restricted Prolog (no function symbols), guaranteed to terminate. Good for relational constraints: reference validity, reachability, uniqueness, name resolution.
  - Seminal: **What You Always Wanted to Know About Datalog** by Ceri, Gottlob, Tanca (IEEE TKDE, 1989).
  - Modern: **Souffle** Datalog engine, used in Doop (Java pointer analysis) and Facebook's Glean.
- **Constraint Handling Rules (CHR)** - multi-headed rules for constraint propagation. You declare constraints and simplification/propagation rules. The engine rewrites constraints until a fixed point.
  - Seminal: **Theory and Practice of Constraint Handling Rules** by Thom Fruhwirth (JLP, 1998).
- **SMT Solvers** (Z3, CVC5) - declare constraints over typed variables (integers, strings, arrays, bitvectors), and the solver checks satisfiability. Effectively Turing-complete in practice.
  - Used by: Dafny, Alloy, Cedar (AWS authorization policy language).
  - Seminal: **Satisfiability Modulo Theories: Introduction and Applications** by de Moura & Bjorner (CACM, 2011).
- **Refinement Types** - types with predicates attached: `{x: Int | x > 0}`. An SMT solver checks that predicates hold at compile time.
  - Seminal: **Liquid Types** by Rondon, Kawaguchi, Jhala (PLDI, 2008).
  - Implementations: Liquid Haskell, F\*.
- **Dependent Types** - types that depend on values. The type system itself becomes Turing-complete. The most expressive option but the hardest to use.
  - Implementations: Idris, Agda, Lean.

::: note
For the dboxide DSL goal:

- **Datalog** for relational constraints (name resolution, reference validity, uniqueness checks).
- **CHR or CUE** for structural validation (field types, required/optional, nesting rules).
- **Refinement types** if you need value-level constraints ("this field must be positive and less than the other field").
:::
