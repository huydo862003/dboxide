# dboxide

![Status](https://img.shields.io/badge/status-active-brightblue)

![DBML](./doc/src/assets/dbml.png)
<a href="https://github.com/huydo862003/Fck-AI-Slop#plan"><img src="https://img.shields.io/badge/human%20slop-90EE90"></a>

A (second & likely not the last 🐱) rewrite of the DBML parser that tries to apply the past experiences with the noobie `@dbml/parse` package.

This is meant to be a companion project alongside [type-theory](https://github.com/hdnax/type-theory).

## Targets

Based on the Rust toolchains (like `salsa` and `rust-analyzer`):

- Research & address certain unclear points related to lexing:
  - What is the ideal way to represent a syntax token? What fields should the token have: source offset/source pointer, token kind, token's processed value (for example, a number for a numeric literal token or an unescaped string for a string literal token)?
  - Assuming UTF-8, how to handle non-ASCII characters?
  - How to conveniently and elegantly handle trivial and error tokens in a lossless syntax tree?
  - How to conveniently and elegantly handle multi-word keywords?
  - How to conveniently and elegantly handle unreserved keywords?
  - How to handle ambiguous tokens? For example, is `<` a less than sign or a left angle bracket as in generic types.
  - How to efficiently store/compute the positions of tokens that can support incremental parsing & syntax tree edits well?
  - Should we use on-demand lexing, instead of lexin all at once?
- Research & address certain unclear points related to parsing:
  - What is a good representation for a lossless syntax tree?
  - Resilience parsing & error recovery techniques.
  - How to design good error messages?
  - Incremental parsing & Red-green tree.
  - How to store error nodes/partial nodes in the lossless syntax tree?
- Program analysis & query-based compiler:
  - How to utilize the query-based architecture for program analysis (and potentially lexing & parsing)?
  - Module system features, such as module resolution.
  - Name resolution & related problems.
- Language server implementation.
- Compilation cancellation.
- Parallelism in compilation?
- SQL dialect awareness.

## Context

The DBML parser was rewritten once around 2023 when I was an intern at Holistics. The original parser was a PEG.js parser.

The main reasons I was assigned to the DBML parser rewrite were (I believe):

- I was an intern at another team that worked solely on the AML language. It was a more complex language so probably, the others thought that a simpler language like DBML was a good task.
- The Peg.js parser had some problems:
  - Slow: I don't think that this is the inherent property of parser combinators. One argument I can come up with to support this idea is that parser combinators tend to have excessive function calls. However, in a compiled language like C++, would this overhead be reduced by inlining or some more sophisticated optimization?
  - Bailing out upon the first error: It didn't implement resilient parsing. Therefore, language services like suggestion is harder to implement, because most of the time when you need suggestion/auto-completion, the source file itself can contain lots of errors.

Since the first version of `@dbml/parse`, there has been some impact, but a lot are left to be desired.

### The Outcome

- Performance: Although `@dbml/parse` was just a naive rewrite, it was already 7 times faster than the Peg.js parser.
- Language services: `@dbml/parse` provides language services like suggestion, go to definition and go to references.
- Resilient parsing: Multiple error messages are allowed. Suggestion works even if the source file is partially broken.

### The Pain

During the first launch, `@dbml/parse` broke a lot of user's code, mainly because the Peg.js parser was too lax that it allowed undocumented/legacy syntax I was not aware of.

Since then, I encountered a lot of pain arising from my poor design choices and the way I wrote tests:

- Fragile snapshot testing: I though snapshots were a shortcut for generating unit tests. This eventually led to me capturing entire CSTs, which created brittle, 2,000+ line test files where trivial internal changes triggered massive diffs. Genuine regression detection was nearly impossible.
- Poor abstraction/Misuse of design patterns: I made the noobie mistake of "the more reuse, the better". I decided to force name resolution and validation phases of the parser into a Template Method pattern. This design choice created tight coupling via a shared component. The base class became a bloated, incomprehensible mess of "hooks" and "configs" to handle slight variations in logic across unrelated components.
- Lack of type-driven validation (Parse, not validate): The parser was too lax, yielding a generic CST rather than a refined IR. Because the CST validation phase didn't transform the data into a "known-good" structure, subsequent phases were forced to re-validate or rely on unsafe type assertions.
- The syntax tokens and nodes positions are precomputed, making CST patches almost always invalidate the positions & incremental parsing partly impossible.

Some are minor issues:

- Error messages & error codes in `@dbml/parse` are a mess.
- No linter setup.
