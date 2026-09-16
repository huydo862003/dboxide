---
_label: "Research Plan"
_icon: icon.document
---

This page outlines the research plan for `dboxide`. The goal is to investigate and understand the following concepts and techniques, which are crucial for building a modern and efficient parser and language server.

## Research Topics

*   **Lexing & Token representation:**
    *   Ideal representation of a syntax token (source offset/pointer, kind, processed value).
    *   Handling of non-ASCII characters (UTF-8).
    *   Handling of trivial and error tokens in a lossless syntax tree.
    *   Handling of multi-word and unreserved keywords.
    *   Handling of ambiguous tokens (e.g., `<` as a less than sign or a generic bracket).
    *   Efficient storage and computation of token positions for incremental parsing.
    *   On-demand lexing vs lexing all at once.
*   **String interning:**
    *   What is string interning and why is it useful in a compiler?
    *   Where and how is it typically applied in a compilation pipeline (e.g., during lexing, parsing, or semantic analysis)?
    *   How can it be implemented efficiently?
    *   What are the trade-offs of using string interning?
*   **Parsing & Syntax tree:**
    *   Good representation for a lossless syntax tree.
    *   Storing error nodes and partial nodes in the lossless syntax tree.
    *   Resilience parsing and error recovery techniques.
    *   Designing good error messages.
*   **Incremental parsing & Red-green trees:**
    *   What are red-green trees and how do they work?
    *   How can they be used to implement incremental parsing?
    *   What are the performance implications of using red-green trees?
*   **Program analysis & Query-based compiler:**
    *   Utilizing a query-based architecture (like `salsa`) for program analysis, lexing, and parsing.
    *   Module system features, including module resolution.
    *   Name resolution and related problems.
    *   **Resources:**
*   **Other topics:**
    *   **Language server implementation**
    *   **Parallelism in compilation**
    *   **SQL dialect awareness**
