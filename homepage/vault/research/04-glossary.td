---
_label: "Glossary"
_icon: icon.book
---

Technical terms and concepts used throughout the `dboxide` project documentation.

## API Boundary

An explicit interface layer that separates different components of a system. By making boundaries opaque (hiding implementation details), you can refactor internal code without breaking external consumers. Example: the separation between the `hir` analysis layer and the `ide` layer in `rust-analyzer`.

## AST

Abstract Syntax Tree. A typed, high-level representation of source code structure that provides ergonomic access to syntactic constructs. In rust-analyzer, AST nodes like `FnDef` and `ParamList` are auto-generated wrappers around the lower-level `SyntaxNode`, offering convenient methods for traversing specific language constructs.

## Block Heuristic

An incremental parsing optimization that reparses only the smallest `{}` block containing an edit. This works because parsers can maintain structurally balanced braces even in broken code by inserting implicit closing braces and wrapping errors. Often not worth the complexity since full reparse is typically fast enough.

## Boolean Blindness

A situation where validation returns a simple boolean (`true`/`false`), causing the system to lose the proof of validity after the check. Instead of returning `bool`, prefer parsing into types that guarantee validity at the type level, preserving the validation evidence throughout the program.

Actually, when writing this, I figured this is related to proof theory's distinction between Admissibility and Derivability (I learned about these in PFPL - see my [type-theory](https://github.com/hdnax/type-theory) repo). Returning `bool` is like admissibility (asserting "this is valid" without proof), while returning a validated type is like derivability (carrying constructive evidence). When you return `bool`, you lose the derivation - just like admissibility is a weaker assertion than derivability because it lacks the constructive proof tree.

## DST

Dynamically Sized Type. A Rust type whose size is not known at compile time. Used in rust-analyzer's GreenNode implementation to store node data and all children in a single heap allocation: `[RefCount | Kind | TextLen | N_Children | Child_1 | ... | Child_N]`.

## Event-Based Parser

A parser architecture where the parser emits abstract events (like `start_node`, `token`, `finish_node`) through trait interfaces (`TokenSource` for input, `TreeSink` for output) rather than directly constructing tree structures. This decouples the parsing algorithm from tree representation, enabling the same parser to work with different input sources and output formats.

## GreenNode

The immutable storage layer in the Red-Green tree architecture. Stores the actual syntax tree data in a space-efficient, persistent structure that can be shared across threads via `Arc`. Contains only `kind: SyntaxKind`, `text_len: usize`, and children, with no position information or parent pointers. Optimizations include DST layout, tagged pointers, and token interning.

## HIR

High-level Intermediate Representation. A fully resolved, semantic view of the code that abstracts away syntactic details. In rust-analyzer, the `hir` crate provides a static API for querying resolved types, names, and semantic relationships, built on top of the syntax tree and incremental analysis passes.

## Incremental Parsing

A technique that reparses only the portions of a file affected by edits rather than reparsing the entire file. Enabled by immutable tree structures (GreenNode) that can be efficiently patched by swapping subtree pointers. Often not worth the implementation complexity as modern parsers are fast enough for full reparse.

## Interning

An optimization technique that stores only one copy of duplicate data. In rust-analyzer, identical tokens (like multiple occurrences of `1` in `1 + 1`) share the same allocated token, reducing memory usage.

## Lossless Syntax Tree

A tree representation that preserves everything from the source code including whitespace, comments, and invalid tokens. Enables perfect reconstruction of the original source text by concatenating token text. Invalid input gets wrapped in `ERROR` nodes rather than being discarded.

## LSP

Language Server Protocol. A standardized protocol for communication between editors and language servers, enabling features like auto-completion, go-to-definition, and diagnostics. Allows one language server implementation to work with multiple editors.

## Monomorphization

The process where Rust's compiler generates separate copies of generic functions for each concrete type used. Can lead to "monomorphization bloat" where excessive use of generics at system boundaries causes long compile times. Mitigated by restricting generics or using dynamic dispatch at boundaries.

## Panic Mode

An error recovery technique where the parser skips tokens until it finds a synchronization point (like `}` or `;`) after encountering a syntax error. Allows the parser to continue and find additional errors rather than stopping at the first one.

## Pratt Parsing

An elegant technique for parsing expressions with operator precedence without building complex precedence tables. Handles infix, prefix, and postfix operators uniformly. Used by rust-analyzer for expression parsing alongside recursive descent for other constructs.

## Red-Green Tree

A tree architecture that separates immutable storage (Green layer) from navigation and position information (Red layer). `GreenNode`s store actual data and can be shared across threads. `SyntaxNode`s (Red layer) are lightweight cursors that add parent pointers and compute positions on-demand. This separation enables efficient tree edits and incremental parsing.

## Recursive Descent Parser

A top-down parsing technique where each grammar rule maps to a function that calls other parsing functions recursively. Makes the parser code easy to understand and debug since it directly mirrors the grammar structure. Used by rust-analyzer for most syntactic constructs.

## Resilient Parsing

A parsing approach that always produces a tree even from syntactically invalid code. Employs error recovery techniques (panic mode, implicit token insertion, early termination) to maintain tree structure. Invalid sections get wrapped in `ERROR` nodes. Enables IDE features like completion to work even in broken code.

## Salsa

An incremental computation framework that enables on-demand, cached evaluation of derived data. When inputs change, Salsa tracks dependencies and recomputes only what's affected. Used in rust-analyzer's `base-db` crate to make analysis incremental and efficient.

## Snapshot Testing

A testing approach where tests are defined as input/output pairs. The framework compares actual output against expected output stored in test files. When behavior changes intentionally, a flag (like `UPDATE_EXPECT=1`) auto-updates expectations, eliminating manual maintenance.

## TokenSource

A trait interface that defines how parsers read input tokens. Provides methods like `current()`, `lookahead_nth()`, and `bump()`. Abstracts over different token sources (source files, macro expansions, synthetic tokens) so the same parser can handle all inputs.

## Tree-Agnostic Parser

A parser that emits abstract parsing events without knowledge of the final tree structure. By working through trait interfaces (TokenSource/TreeSink), the parser remains decoupled from tree representation, enabling the same parsing logic to produce different output formats.

## TreeSink

A trait interface that defines how parsers write output. Receives events like `start_node()`, `token()`, `finish_node()`, and `error()`. The implementation decides how to build the actual tree structure. This abstraction enables different tree formats from the same parser.

## Trivia

Non-semantic tokens like whitespace, comments, and formatting characters. Different parsers handle trivia differently: explicit nodes (trivia as sibling nodes), attached trivia (stored in leading/trailing properties of tokens), or linked lists (separate doubly-linked list alongside the tree). rust-analyzer uses explicit nodes for uniformity and losslessness.
