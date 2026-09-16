---
_label: "Research Summary"
_icon: icon.document
---

## Operations

1. Study established projects to inherit their mature operational workflows [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
2. Convert recurring code review feedback into a permanent "living style guide" that scales mentorship and prevents repetitive corrections [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
3. Structure commit metadata (titles and messages) to automate downstream documentation like changelogs, which reduces manual release overhead [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).

## Code Evolution

1. Classify changes by their impact scope (internal vs. API/dependency) to apply proportional review scrutiny [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
2. Internalize trivial utilities ("micro-dependencies") to reduce supply chain attack surface and compilation bloat [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
3. Prefer parsing data into types that *guarantee* validity over simple validation checks, which eliminates "boolean blindness" where the system loses the proof of validity after the check returns `true` [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
4. Restrict heavy generics at system boundaries to prevent "monomorphization bloat," trading minor runtime overhead for significant compile-time gains [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).

## Compiler Architecture

`rust-analyzer`'s high-level architecture, from low-level to higher-level [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions):

1. `parser` crate: A recursive descent parser that is tree-agnostic and event-based.
2. `syntax` crate: Syntax tree structure and parser that exposes a higher-level API for the code's syntactic structure.
3. `base-db` crate: Integration with `salsa`, allowing for incremental and on-demand computation.
4. `hir-xxx` crates: Program analysis phases that explicitly integrate incremental computation.
5. `hir` crate: A high-level API that provides a static, inert and fully resolved view of the code.
6. `ide` crate: Provides high-level IDE features on top of the `hir` semantic model, speaking IDE languages such as texts and offsets instead of syntax nodes.
7. `rust-analyzer`: The language server that knows about LSP and JSON protocols.

## Token & Node

1. Parser tokens have tags and their corresponding source text, while parser nodes have tags and source length with children nodes placed in a homogenous vector [2](02-resources/01-rust-analyzer/03-syntax-tree-and-parser).
2. Tokens and nodes share the same `SyntaxKind` enum and are not as clearly distinguished as in `@dbml/parse`. Tokens function as leaf nodes while nodes function as interior nodes [2](02-resources/01-rust-analyzer/03-syntax-tree-and-parser).
3. The [explicit nodes](02-resources/01-rust-analyzer/03-syntax-tree-and-parser#dealing-with-trivia) approach treats whitespace and comments as sibling nodes, where `rust-analyzer` handles trivial and error nodes uniformly (everything is a node), while some parsers like `@dbml/parse` attach trivia to semantic parents and others use hybrid approaches [2](02-resources/01-rust-analyzer/03-syntax-tree-and-parser).
4. For context-sensitive keywords like `union` and `default`, the parser checks the actual text via `TokenSource::is_keyword()` rather than relying solely on token kind [2](02-resources/01-rust-analyzer/03-syntax-tree-and-parser).
5. An intermediary layer using `TokenSource` and `TreeSink` traits merges tokens based on context, such as combining `>` + `>` into `>>` [2](02-resources/01-rust-analyzer/03-syntax-tree-and-parser).
6. Nodes store only `text_len` (not absolute offset) while tokens store text, and SyntaxNode computes offset on-demand from its parent, which enables incremental parsing without position invalidation [2](02-resources/01-rust-analyzer/03-syntax-tree-and-parser).

## Parsing

1. The three-layer tree architecture separates concerns effectively [2](02-resources/01-rust-analyzer/03-syntax-tree-and-parser):
    - [GreenNode (storage)](02-resources/01-rust-analyzer/03-syntax-tree-and-parser#layer-1-greennode-the-storage): An immutable, persistent layer with optimizations including DST (single heap allocation), tagged pointers, token interning, and `Arc`-sharing that stores `text_len` rather than offset.
    - [SyntaxNode (cursor/RedNode)](02-resources/01-rust-analyzer/03-syntax-tree-and-parser#layer-2-syntaxnode-the-cursor--rednode): Adds parent pointers and on-demand position computation where memory scales with traversal depth rather than tree size, and nodes are transient (rebuilt from GreenNode when needed).
    - [AST (typed API)](02-resources/01-rust-analyzer/03-syntax-tree-and-parser#layer-3-ast-the-api): Auto-generated typed wrappers like `FnDef` and `ParamList` around `SyntaxNode` that provide ergonomic access to specific constructs.
2. Everything is preserved including whitespace, comments, and invalid tokens, where invalid input gets wrapped in `ERROR` nodes and the original text can be reconstructed by concatenating token text [2](02-resources/01-rust-analyzer/03-syntax-tree-and-parser).
3. Errors live in a separate `Vec<SyntaxError>` rather than being embedded in the tree, which enables manual tree construction without error state management and produces parser output as `(green_node, errors)` [2](02-resources/01-rust-analyzer/03-syntax-tree-and-parser).
4. Resilient parsing combines multiple strategies [2](02-resources/01-rust-analyzer/03-syntax-tree-and-parser):
    - The algorithm uses recursive descent with Pratt parsing for expressions and is intentionally permissive, accepting invalid constructs that are validated later.
    - The [event-based architecture](02-resources/01-rust-analyzer/03-syntax-tree-and-parser#parsing---the-token-sequence-transformer) has the parser emit abstract events via `TokenSource` (input) and `TreeSink` (output) traits, keeping the parser agnostic to tree structure.
    - Error recovery employs panic mode (skipping to synchronization points like `}` or `;`), inserts implicit closing braces, performs early block termination, and wraps malformed content in ERROR nodes.
5. [Incremental parsing](02-resources/01-rust-analyzer/03-syntax-tree-and-parser#incremental-reparse) uses a sophisticated approach [2](02-resources/01-rust-analyzer/03-syntax-tree-and-parser):
    - The Red-Green model separates Green (immutable storage) from Red (cursors with positions), where this separation enables cheap tree patches by swapping GreenNode pointers.
    - The [block heuristic](02-resources/01-rust-analyzer/03-syntax-tree-and-parser#incremental-reparse) reparses only the smallest `{}` block containing the edit, which works because the parser maintains structurally balanced braces through implicit `}` insertion and ERROR wrapping for extras.
    - Pragmatically, incremental reparsing is often not worth the complexity since full reparse is fast and simpler, though the architecture remains valuable for tree edit cheapness and subtree sharing.
6. [Error messages](02-resources/01-rust-analyzer/03-syntax-tree-and-parser#reporting-syntax-errors) use a layered approach where a permissive parser is followed by a separate validation pass for "soft" errors, allowing the parser to focus on structure recovery while validation uses semantic context for detailed diagnostics [2](02-resources/01-rust-analyzer/03-syntax-tree-and-parser).

## Design Choices & General Architectures

1. Avoid blind serialization of internal types, which implicitly couples public clients to private implementation details [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
2. Enforce opaque API boundaries (such as between analysis and IDE layers) to enable radical internal refactoring without breaking consumers [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
3. Codify architectural laws (like "core layers are I/O free") to permanently guarantee non-functional requirements such as speed and deterministic testing [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
4. Order function arguments by stability (context -> data) to align code structure with mental models ("setting" -> "actors") and reduce cognitive load during scanning [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
5. Use distinct types to segregate unverified external input (like "dirty" OS strings) from validated internal data, preventing logic errors from crossing trust boundaries [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
6. Enforce invariants via "construction & retrieval" (private fields with public getters) rather than "mutation" (setters), ensuring objects never enter invalid states [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
7. Encode assumptions into the type system (such as non-nullable types) to force callers to handle edge cases explicitly, preserving context at the call site [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
8. Encapsulate complex execution arguments into temporary structs to support multiple execution modes without duplicating function signatures [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
9. Split functions with boolean flags (like `do(true)`) into distinct named functions, which adheres to the Single Responsibility Principle and prevents unrelated logic paths from coupling [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).

### Implementation Patterns

1. Prioritize imperative clarity over functional brevity, where code should maximize "work per line" rather than minimizing line count via complex indirections [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
2. Use spatial operators (like `<` or `<=`) that map intuitively to the mental number line (`0->infinity`), avoiding the mental effort required to "flip" comparisons [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
3. Prefer syntax that supports left-to-right reading (such as explicit type ascription), which reduces the "context window" required to understand a statement by declaring intent up-front [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
4. Use blocks `{ ... }` to isolate temporary state, preventing variable pollution while retaining access to the parent context [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
5. Push resource allocation (memory and I/O) up to the call site (like passing a buffer in) to make performance costs visible and controllable by the caller [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
6. Use explicit namespaces or qualifiers to visually reinforce layer boundaries in code, such as distinguishing `ast::Node` from `hir::Node` [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).

## Testing

1. Tests are defined as input/output pairs where the framework compares actual results against expected output, and expectations are updated when behavior changes intentionally [3](02-resources/01-rust-analyzer/04-tests).
2. Define tests via data (input and output) rather than API calls, which makes tests survive refactoring [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
3. Each test simulates a complete environment (multi-file, multi-crate) in memory without shared state [3](02-resources/01-rust-analyzer/04-tests).
4. Failing tests can auto-update their expected output via a flag, which eliminates manual maintenance [3](02-resources/01-rust-analyzer/04-tests).
5. Minimize test cases to the smallest input that reproduces the behavior, resulting in less noise and faster debugging [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
6. Place tests near their implementation to enable easy discovery during development [3](02-resources/01-rust-analyzer/04-tests).
7. Never ignore test failures. Instead, assert the incorrect behavior with a FIXME comment to keep the failure visible [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).

## Documentation

Enforce full-sentence comments (starting with a capital letter and ending with a period) to psychologically shift the author from "note-taking" (describing what) to "explanation" (describing why and providing context) [1](02-resources/01-rust-analyzer/01-high-level-architecture-and-conventions).
