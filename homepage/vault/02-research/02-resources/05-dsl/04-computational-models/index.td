---
_label: "Alternative Computational Models for Validation"
_icon: icon.sun
---

Resources on non-imperative models for data validation, from restricted logic to fully Turing-complete type systems.

::: note
For the dboxide DSL goal:

- **Datalog** for relational constraints (name resolution, reference validity, uniqueness checks).
- **CHR or CUE** for structural validation (field types, required/optional, nesting rules).
- **Refinement types** if you need value-level constraints ("this field must be positive and less than the other field").
:::

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
