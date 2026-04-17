# Introduction

<p align="center">
  <img src="assets/dbml.png" alt="dboxide logo" width="200"/>
</p>

This repository, [`dboxide`](https://github.com/hdnax/dboxide), is a rewrite of a DBML parser. It is a learning project to explore concepts in language theory, inspired by the work in the [`type-theory`](https://github.com/hdnax/type-theory) repository. The project builds on lessons learned from a previous parser, `@dbml/parse`.

The primary motivation for this rewrite is to address the shortcomings of previous versions and explore more advanced compiler design concepts. The original PEG.js parser was slow and lacked resilience, making it difficult to build a good developer experience (e.g., in a language server). The first rewrite, `@dbml/parse`, brought performance improvements and basic language services but suffered from design flaws that made it fragile and hard to maintain.

This project serves to be both about parsing DBML & a research ground for:

- **Compiler infrastructure:** Investigating best practices for lexing, parsing, and representing code in a way that supports incremental updates and rich analysis.
- **Developer experience:** Building a foundation for a high-quality language server with features like precise error reporting, autocompletion, and code navigation.
- **Modern techniques:** Applying query-based (on-demand) compilation, inspired by Salsa, to create a scalable and efficient analysis engine.

The goal is to create a parser that is not only fast and correct but also serves as a solid foundation for a new generation of DBML tooling.
