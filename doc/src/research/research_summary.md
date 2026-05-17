# Research Summary

## Operations

1. Study established projects to inherit their mature operational workflows<sup>[1](#1)</sup>.
2. Convert recurring code review feedback into a permanent "living style guide" that scales mentorship and prevents repetitive corrections<sup>[1](#1)</sup>.
3. Structure commit metadata (titles and messages) to automate downstream documentation like changelogs, which reduces manual release overhead<sup>[1](#1)</sup>.

## Code Evolution

1. Classify changes by their impact scope (internal vs. API/dependency) to apply proportional review scrutiny<sup>[1](#1)</sup>.
2. Internalize trivial utilities ("micro-dependencies") to reduce supply chain attack surface and compilation bloat<sup>[1](#1)</sup>.
3. Prefer parsing data into types that *guarantee* validity over simple validation checks, which eliminates "boolean blindness" where the system loses the proof of validity after the check returns `true`<sup>[1](#1)</sup>.
4. Restrict heavy generics at system boundaries to prevent "monomorphization bloat," trading minor runtime overhead for significant compile-time gains<sup>[1](#1)</sup>.

## Compiler Architecture

`rust-analyzer`'s high-level architecture, from low-level to higher-level<sup>[1](#1)</sup>:

1. `parser` crate: A recursive descent parser that is tree-agnostic and event-based.
2. `syntax` crate: Syntax tree structure and parser that exposes a higher-level API for the code's syntactic structure.
3. `base-db` crate: Integration with `salsa`, allowing for incremental and on-demand computation.
4. `hir-xxx` crates: Program analysis phases that explicitly integrate incremental computation.
5. `hir` crate: A high-level API that provides a static, inert and fully resolved view of the code.
6. `ide` crate: Provides high-level IDE features on top of the `hir` semantic model, speaking IDE languages such as texts and offsets instead of syntax nodes.
7. `rust-analyzer`: The language server that knows about LSP and JSON protocols.

## Token & Node

1. Parser tokens have tags and their corresponding source text, while parser nodes have tags and source length with children nodes placed in a homogenous vector<sup>[2](#2)</sup>.
2. Tokens and nodes share the same `SyntaxKind` enum and are not as clearly distinguished as in `@dbml/parse`. Tokens function as leaf nodes while nodes function as interior nodes<sup>[2](#2)</sup>.
3. The [explicit nodes](./resources/rust-analyzer/syntax_tree_and_parser.md#dealing-with-trivia) approach treats whitespace and comments as sibling nodes, where `rust-analyzer` handles trivial and error nodes uniformly (everything is a node), while some parsers like `@dbml/parse` attach trivia to semantic parents and others use hybrid approaches<sup>[2](#2)</sup>.
4. For context-sensitive keywords like `union` and `default`, the parser checks the actual text via `TokenSource::is_keyword()` rather than relying solely on token kind<sup>[2](#2)</sup>.
5. An intermediary layer using `TokenSource` and `TreeSink` traits merges tokens based on context, such as combining `>` + `>` into `>>`<sup>[2](#2)</sup>.
6. Nodes store only `text_len` (not absolute offset) while tokens store text, and SyntaxNode computes offset on-demand from its parent, which enables incremental parsing without position invalidation<sup>[2](#2)</sup>.

## Parsing

1. The three-layer tree architecture separates concerns effectively<sup>[2](#2)</sup>:
    - [GreenNode (storage)](./resources/rust-analyzer/syntax_tree_and_parser.md#layer-1-greennode-the-storage): An immutable, persistent layer with optimizations including DST (single heap allocation), tagged pointers, token interning, and `Arc`-sharing that stores `text_len` rather than offset.
    - [SyntaxNode (cursor/RedNode)](./resources/rust-analyzer/syntax_tree_and_parser.md#layer-2-syntaxnode-the-cursor--rednode): Adds parent pointers and on-demand position computation where memory scales with traversal depth rather than tree size, and nodes are transient (rebuilt from GreenNode when needed).
    - [AST (typed API)](./resources/rust-analyzer/syntax_tree_and_parser.md#layer-3-ast-the-api): Auto-generated typed wrappers like `FnDef` and `ParamList` around `SyntaxNode` that provide ergonomic access to specific constructs.
2. Everything is preserved including whitespace, comments, and invalid tokens, where invalid input gets wrapped in `ERROR` nodes and the original text can be reconstructed by concatenating token text<sup>[2](#2)</sup>.
3. Errors live in a separate `Vec<SyntaxError>` rather than being embedded in the tree, which enables manual tree construction without error state management and produces parser output as `(green_node, errors)`<sup>[2](#2)</sup>.
4. Resilient parsing combines multiple strategies<sup>[2](#2)</sup>:
    - The algorithm uses recursive descent with Pratt parsing for expressions and is intentionally permissive, accepting invalid constructs that are validated later.
    - The [event-based architecture](./resources/rust-analyzer/syntax_tree_and_parser.md#parsing---the-token-sequence-transformer) has the parser emit abstract events via `TokenSource` (input) and `TreeSink` (output) traits, keeping the parser agnostic to tree structure.
    - Error recovery employs panic mode (skipping to synchronization points like `}` or `;`), inserts implicit closing braces, performs early block termination, and wraps malformed content in ERROR nodes.
5. [Incremental parsing](./resources/rust-analyzer/syntax_tree_and_parser.md#incremental-reparse) uses a sophisticated approach<sup>[2](#2)</sup>:
    - The Red-Green model separates Green (immutable storage) from Red (cursors with positions), where this separation enables cheap tree patches by swapping GreenNode pointers.
    - The [block heuristic](./resources/rust-analyzer/syntax_tree_and_parser.md#incremental-reparse) reparses only the smallest `{}` block containing the edit, which works because the parser maintains structurally balanced braces through implicit `}` insertion and ERROR wrapping for extras.
    - Pragmatically, incremental reparsing is often not worth the complexity since full reparse is fast and simpler, though the architecture remains valuable for tree edit cheapness and subtree sharing.
6. [Error messages](./resources/rust-analyzer/syntax_tree_and_parser.md#reporting-syntax-errors) use a layered approach where a permissive parser is followed by a separate validation pass for "soft" errors, allowing the parser to focus on structure recovery while validation uses semantic context for detailed diagnostics<sup>[2](#2)</sup>.

## Design Choices & General Architectures

1. Avoid blind serialization of internal types, which implicitly couples public clients to private implementation details<sup>[1](#1)</sup>.
2. Enforce opaque API boundaries (such as between analysis and IDE layers) to enable radical internal refactoring without breaking consumers<sup>[1](#1)</sup>.
3. Codify architectural laws (like "core layers are I/O free") to permanently guarantee non-functional requirements such as speed and deterministic testing<sup>[1](#1)</sup>.
4. Order function arguments by stability (context -> data) to align code structure with mental models ("setting" -> "actors") and reduce cognitive load during scanning<sup>[1](#1)</sup>.
5. Use distinct types to segregate unverified external input (like "dirty" OS strings) from validated internal data, preventing logic errors from crossing trust boundaries<sup>[1](#1)</sup>.
6. Enforce invariants via "construction & retrieval" (private fields with public getters) rather than "mutation" (setters), ensuring objects never enter invalid states<sup>[1](#1)</sup>.
7. Encode assumptions into the type system (such as non-nullable types) to force callers to handle edge cases explicitly, preserving context at the call site<sup>[1](#1)</sup>.
8. Encapsulate complex execution arguments into temporary structs to support multiple execution modes without duplicating function signatures<sup>[1](#1)</sup>.
9. Split functions with boolean flags (like `do(true)`) into distinct named functions, which adheres to the Single Responsibility Principle and prevents unrelated logic paths from coupling<sup>[1](#1)</sup>.

### Implementation Patterns

1. Prioritize imperative clarity over functional brevity, where code should maximize "work per line" rather than minimizing line count via complex indirections<sup>[1](#1)</sup>.
2. Use spatial operators (like `<` or `<=`) that map intuitively to the mental number line (`0->infinity`), avoiding the mental effort required to "flip" comparisons<sup>[1](#1)</sup>.
3. Prefer syntax that supports left-to-right reading (such as explicit type ascription), which reduces the "context window" required to understand a statement by declaring intent up-front<sup>[1](#1)</sup>.
4. Use blocks `{ ... }` to isolate temporary state, preventing variable pollution while retaining access to the parent context<sup>[1](#1)</sup>.
5. Push resource allocation (memory and I/O) up to the call site (like passing a buffer in) to make performance costs visible and controllable by the caller<sup>[1](#1)</sup>.
6. Use explicit namespaces or qualifiers to visually reinforce layer boundaries in code, such as distinguishing `ast::Node` from `hir::Node`<sup>[1](#1)</sup>.

## Testing

1. Tests are defined as input/output pairs where the framework compares actual results against expected output, and expectations are updated when behavior changes intentionally<sup>[3](#3)</sup>.
2. Define tests via data (input and output) rather than API calls, which makes tests survive refactoring<sup>[1](#1)</sup>.
3. Each test simulates a complete environment (multi-file, multi-crate) in memory without shared state<sup>[3](#3)</sup>.
4. Failing tests can auto-update their expected output via a flag, which eliminates manual maintenance<sup>[3](#3)</sup>.
5. Minimize test cases to the smallest input that reproduces the behavior, resulting in less noise and faster debugging<sup>[1](#1)</sup>.
6. Place tests near their implementation to enable easy discovery during development<sup>[3](#3)</sup>.
7. Never ignore test failures. Instead, assert the incorrect behavior with a FIXME comment to keep the failure visible<sup>[1](#1)</sup>.

## Documentation

Enforce full-sentence comments (starting with a capital letter and ending with a period) to psychologically shift the author from "note-taking" (describing what) to "explanation" (describing why and providing context)<sup>[1](#1)</sup>.

## References

1. [`rust-analyzer` high-level architecture and conventions](https://www.google.com/search?q=../research/resources/rust-analyzer/high_level_architecture_and_conventions.md) <a id="1"></a>
2. [`rust-analyzer` syntax tree and parser](./resources/rust-analyzer/syntax_tree_and_parser.md) <a id="2"></a>
3. [`rust-analyzer` testing](./resources/rust-analyzer/tests.md) <a id="3"></a>
