---
_type: Book
_label: "Alternative Computational Models"
author: "Martin Fowler"
date: "2010"
description: "Non-imperative models that pair well with DSLs"
isbn: "978-0321712943"
url: "https://martinfowler.com/books/dsl.html"
tags:
  - "dsl"
  - "computational model"
---

::: tip
- Alternative computational models are one of the strongest reasons to use a DSL.
- If your problem fits imperative code naturally, a regular language works fine.
- The DSL benefits (productivity, domain expert communication) really kick in when the underlying model is non-imperative.
:::

## Why Alternative Models Matter

The imperative model is good at expressing **implementation** (how the program works), but often poor at expressing **intent** (what the program is trying to achieve).

When intent does not map to a sequence of steps, an alternative computational model can express it more directly.

- An adaptive model captures intent as data (object configuration). The DSL makes that configuration readable.
  - The big downside is that behavior becomes implicit.
    - You cannot just read the code to understand what happens.
    - You have to trace through the model's instance configuration.
- A DSL helps here by giving you something readable to look at, even if it does not fully remove the comprehension problem.

::: tip
Always provide a tracing mechanism so users can see which rules fired and in what order.
:::

## A Few Alternative Models

This section outlines some alternative computational model other than the normal imperative model.

### Decision Table

A table of conditions (rows) and consequences (rows). Each column is a rule: match the conditions, apply the consequences.

| | Col 1 | Col 2 | Col 3 |
|---|---|---|---|
| Premium Customer | Y | Y | N |
| Priority Order | Y | N | Y |
| Fee | 150 | 100 | 70 |
| Alert Rep | Y | Y | N |

- Easy for non-programmers to follow. The tabular layout maps naturally to spreadsheets, making it one of the few cases where domain experts can directly edit the DSL.
- The imperative equivalent (nested if/else) forces an execution order that the decision table does not imply. The table can also check for completeness (missing permutations) automatically.

### Production Rule System

You write individual rules, each with a **condition** and an **action**.

- _Unlike normal if/else_, you do not specify the order of evaluation.
- The system decides which rules to fire and in what order based on the current state of the data.

```
if passenger.frequentFlier
then passenger.priorityHandling = true;

if mileage > 25000
then passenger.frequentFlier = true;
```

In imperative code, the order you write these matters. If you check `frequentFlier` before setting it, the first rule never fires.

In a Production Rule System, the engine re-evaluates after each rule fires. Rule 2 sets `frequentFlier = true`, which makes rule 1's condition true, so rule 1 fires next. This is called **chaining**.

- Pros:
  - You write rules individually without worrying about order or interactions. The system figures out the consequences.
  - Adding a new rule does not require rethinking the control flow of existing rules.
- Cons:
  - Chaining makes behavior implicit. With many rules, chain reactions become hard to predict (rule A triggers rule B which disables rule C).
  - A tracing mechanism is essential so you can see which rules fired and why.

A Decision Table can be seen as a Production Rule System without chaining, where each column is one independent rule.

::: note
Production Rule Systems and Dependency Networks are both forms of **forward chaining inference**. You declare facts and rules, and the engine derives consequences automatically. Production Rule Systems chain by re-evaluating rule conditions after each action. Dependency Networks chain by propagating "needs to be done" signals through the dependency graph. Both share the same tradeoff: declarative and composable, but harder to trace when things go wrong.
:::

### State Machine

Divides the behavior of an object into a finite set of states. Events trigger transitions between states, and entering a state can trigger actions. The running example throughout this book (Miss Grant's controller) is a state machine.

- A common choice because many systems can be thought of as reacting to events by moving through states (order processing, UI workflows, protocol handlers).
- Many variations exist around how actions are triggered (on entry, on exit, on transition) and how hierarchies of states are handled (statecharts).

### Dependency Network

Defines tasks and their prerequisites. The system figures out execution order and avoids redundant work.

```
test --> compile, load-data
compile --> generate-code
load-data --> generate-code
```

- Build tools (Make, Ant, Rake) are the most familiar example.
- A good fit when you have computationally expensive tasks with dependencies between them.

## Choosing a Model

- Try it on paper first with simple text and diagrams. If it passes a desk check, prototype it.
- Focus on getting the semantic model working before polishing the DSL. Once the model is solid, experimenting with different DSL syntaxes is cheap.
