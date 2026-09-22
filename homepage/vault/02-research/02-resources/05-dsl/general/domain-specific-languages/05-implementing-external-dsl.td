---
_type: Book
_label: "Implementing an External DSL"
author: "Martin Fowler"
date: "2010"
description: "Techniques for building external DSLs: parsing strategies, grammar concepts, XML DSLs"
isbn: "978-0321712943"
url: "https://martinfowler.com/books/dsl.html"
tags:
  - "dsl"
  - "external dsl"
  - "parsing"
---

This chapter covers how to implement an external DSL:

1. Choosing a parsing strategy.
2. Understanding grammar theory.
3. Deciding on output production.

## Syntactic Analysis Strategy

- **Delimiter-Directed Translation**: split input into lines, process each based on a leading keyword.
  - Pros: uses familiar tools (string splitting, regex).
  - Cons: cannot handle hierarchic structure. The more nesting you need, the more manual work.
- **Syntax-Directed Translation**: define a formal grammar, then use it to drive the parse.
  - Pros: handles hierarchy naturally.
  - Cons: steeper learning curve, especially with Parser Generator tooling.
  - Three ways to turn a grammar into a parser:
    - **Recursive Descent Parser**: each grammar rule becomes a function.
      - Pros: easy to understand.
      - Cons: the grammar gets lost in control flow.
    - **Parser Combinator**: each rule becomes an object you compose.
      - Pros: represents the grammar explicitly in code.
      - Cons: not quite as clear as true BNF.
    - **Parser Generator**: write the grammar in BNF, and the tool generates a parser.
      - Pros: most mature and powerful. Grammar is clearly defined.
      - Cons: takes time to learn, complicates the build process.

## Output Production Strategy

- **Embedded Translation**: populate the semantic model directly during parsing. As soon as you recognize a construct, create the model object.
- **Tree Construction**: first build an explicit syntax tree, then walk it in a second pass to populate the semantic model.

::: tip
The more distance between the DSL syntax and the semantic model structure, the more useful Tree Construction becomes. For simple DSLs, Embedded Translation is less work.
:::

## Parsing Concepts

**Separated Lexing**: Syntax-Directed Translation splits into two stages. Lexing converts text to tokens, then syntactic analysis arranges tokens into a tree.

```
state idle  -->  [{type: state-keyword, content: "state"},
                  {type: identifier, content: "idle"}]
```

Lexing is usually done with a **Regex Table Lexer**: a list of regex-to-token-type rules applied in order. Two things to watch for:

- **Keywords vs identifiers**: `state initial state` is ambiguous because the lexer classifies the second "state" as a keyword. You need **Alternative Tokenization** to handle this.
- **Syntactic whitespace** (newlines as separators, indentation as structure) complicates lexing significantly.

**Chomsky Hierarchy**: grammars are classified into three levels that matter for DSLs:

- **Regular**: processable with a finite-state machine. Cannot handle nesting (parentheses, nested blocks).
- **Context-free**: adds a stack, allowing nesting. Most programming languages and DSLs are context-free.
- **Context-sensitive**: meaning depends on surrounding context (e.g. a variable must be declared before use). We cannot generate parsers for these, which is why we use symbol tables instead.

If your DSL has nested blocks, you need at least a context-free parser and Syntax-Directed Translation.

**Top-Down vs Bottom-Up Parsing**:

- **Top-down** (LL): starts from the highest grammar rule and works down. Recursive Descent, Parser Combinators, and ANTLR are all top-down. Cannot handle left recursion (`expr: expr '+' expr`).
- **Bottom-up** (LR): shifts tokens onto a stack and reduces when a rule matches. More powerful but harder to debug. Yacc is the classic example.

For DSLs, top-down parsers are almost always sufficient.

## Foreign Code

When a DSL needs to handle rare special cases, embed a general-purpose language instead of growing the DSL.

```
scott handles floor_wax in MA RI CT when {/^Baker/.test(lead.name)};
```

The code inside `{}` is slurped as a string by the lexer (using Alternative Tokenization) and stored in the semantic model for later execution.

## XML as Carrier Syntax

- XML provides schema validation, consistent escaping, and off-the-shelf parsers. A custom syntax gives you none of this for free.
- The tradeoff is syntactic noise. Angle brackets, closing tags, and quoting obscure the DSL content.
- JSON and YAML carry less noise but are data serialization formats. A DSL needs fluency, not just structured data.
