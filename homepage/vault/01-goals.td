---
_label: "Goals"
_icon: icon.target
---

This document outlines the primary goals for `dboxide`. These goals are divided into two main categories: **Project Goals**, which focus on the tangible outcomes of the project, and **Learning Goals**, which focus on the practical techniques and concepts to be explored during development.

## Project Goals

These goals are about building a high-quality, feature-rich, and performant DBML parser and associated tooling.

*   **Resilient and high-quality parser:** Create a parser that can gracefully handle syntax errors, provide meaningful error messages, and produce a lossless syntax tree. This is crucial for a good developer experience.
*   **Language server implementation:** Build a functional language server for DBML that provides features like:
    *   Autocompletion
    *   Go to definition
    *   Find references
    *   Real-time diagnostics
*   **Performance:** The parser should be significantly faster than previous implementations, leveraging the performance benefits of Rust.
*   **SQL dialect awareness:** The parser should be able to understand and handle different SQL dialects, making it more versatile.
*   **Modularity and extensibility:** Design the parser and its components in a way that is easy to maintain, extend, and reuse.

## Learning Goals

This project is also an opportunity to learn and apply modern compiler construction techniques. The focus is on understanding the "how" and "why" behind these techniques.

*   **Lexing and parsing techniques:**
    *   **Token representation:** Investigate the ideal way to represent syntax tokens, including their position, kind, and value.
    *   **Lossless syntax trees:** Learn how to build and work with lossless syntax trees that preserve all source information, including whitespace and comments.
    *   **Resilient parsing:** Master techniques for error recovery and resilience in parsers.
    *   **Incremental parsing and red-green trees:** Understand and implement incremental parsing using red-green trees to efficiently re-parse only the changed parts of a file.
*   **Query-based compilation:**
    *   **Salsa framework:** Learn how to use the `salsa` framework to build a query-based compiler architecture.
    *   **Incremental computation:** Understand how query-based systems enable incremental computation, where only the necessary computations are re-run when the input changes.
    *   **Parallelism:** Explore opportunities for parallelizing the compilation process to improve performance.
* **Type-driven validation:** Learn to apply the "Parse, don't validate" principle to create a more robust and type-safe parser.
* **Effective testing strategies:** Learn how to write effective and maintainable unit and integration tests for a compiler.
