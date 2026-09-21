---
_type: Concept
_label: "Class Symbol Table"
status: 'learning'
tags:
  - 'internal dsl'
  - 'dsl pattern'
depends_on:
  - fref("02-research/04-glossary/symbol-table.td")
  - fref("02-research/04-glossary/expression-builder.td")
---

Fowler's term for using the host language's type system as a symbol table for an internal DSL. Each symbol type (states, events, commands) is defined as a class, and the symbols used in a DSL script are declared as fields on that class. This gives IDE autocompletion and compile-time type checking for DSL scripts, instead of relying on raw strings.

```java
class BasicStateMachine extends StateMachineBuilder {
    States idle, active, waitingForLight;
    Events doorClosed, drawerOpened, lightOn;
    Commands unlockPanel, lockDoor;
}
```
