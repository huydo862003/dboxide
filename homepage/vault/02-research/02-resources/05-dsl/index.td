---
_label: "General DSL Philosophy"
_icon: icon.sun
---

Resources about DSL design philosophy relevant to dboxide as a whole. dboxide's DSL research has two parts: a **structural DSL** for declaratively matching syntax, resolving names, and checking types, and a **logic DSL** for composable constraint languages and validation.

- **Domain-Specific Languages** by Martin Fowler (Addison-Wesley, 2010). The definitive catalog of DSL patterns: internal vs external, semantic models, code generation.
- **The Racket Manifesto** by Felleisen et al. (SNAPL 2015).
  - Articulates language-oriented programming and self-hosting DSL design.
  - Link: [PDF](https://www2.ccs.neu.edu/racket/pubs/manifesto.pdf)
- **The Evolution of Lua** by Ierusalimschy et al. (HOPL III, 2007).
  - How a config language evolved into a turing-complete embeddable language.
  - Link: [PDF](https://www.cs.tufts.edu/comp/250RTS/archive/roberto-ierusalimschy/lua-hopl-iii.pdf)
- **Crafting Interpreters** by Bob Nystrom (this one already read, but may come back useful).
  - Two complete language implementations: tree-walking interpreter and bytecode VM.
  - Covers parsing, scoping, closures, and value representation.
  - Link: [craftinginterpreters.com](https://craftinginterpreters.com/)
