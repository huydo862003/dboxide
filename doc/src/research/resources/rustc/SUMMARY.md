# Rustc Development Guide

Basically the [rust-analyzer](../rust-analyzer/SUMMARY.md) has covered most of AST design & query-based compilation. However, it (seems to) only cover incrementalism inside a process. What if I want incrementalism to survive across recompilations? Rustc as a compiler would really want this, so it covers the following:

- [Incremental Compilation in Detail](./incremental_compilation.md)
- [Cache Persistence](./cache_persistence.md)
- [StableHash Infrastructure](./stable_hash.md)
- [Glossary](./glossary.md)
