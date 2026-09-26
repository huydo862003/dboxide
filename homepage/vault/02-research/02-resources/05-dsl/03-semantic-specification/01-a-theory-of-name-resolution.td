---
_type: Book
_label: "A Theory of Name Resolution"
url: "https://link.springer.com/chapter/10.1007/978-3-662-46669-8_9"
author: "Neron et al."
date: "2015"
tags:
  - "name resolution"
  - "scope graphs"
  - "type system"
---

**Scope graphs** as a **language-independent** **formalism** for **name binding**.

- Language-independent: The formalism can express name binding for a wide range of languages.
- Formalism: A formal mathematical model.
- Name binding: Express declaration and references of entities via name.

PDF link: https://eelcovisser.org/publications/2015/NeronTVW15.pdf

## Context: Spoofax

Spoofax is a ${fref("02-research/04-glossary/language-workbench.td")}.

A **language workbench** is a tool for defining a DSL together with its editing environment.

This paper provides:

- **The part of the theoretical foundation behind Spoofax's name resolution engine**.
- **Scope graphs**, which were developed to give Spoofax a _declarative, language-independent_ model for **name binding** (that could be used to drive IDE features like go-to-definition and find-references).

## Abstract

- A theory that is:
  - **Language-independent**
  - For **name-binding and resolution**
  - Suitable for programming languages with various **scoping rules** including **lexical scoping** and **modules**.
- Modeling **name resolution** as a **2-step problem**:
  1. Construct a **scope graph** from **language-specific** ASTs using **language-specific** rules.
  2. References in the scope graph are resolved to corresponding declarations using a **language-independent** resolution process.
- Devise:
  - A **resolution calculus**: a concise, declarative, language-independent specification of name resolution.
  - A **resolution algorithm** that is sound and complete with respect to the calculus.
  - Language-independent definitions of:
    - **alpha-equivalence**
    - **rename refactoring**
    from the calculus.
- Shows the approach covers a range of name binding patterns found in existing languages.
