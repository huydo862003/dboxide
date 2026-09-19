---
_type: Book
_label: "An Introductory Example"
author: "Martin Fowler"
date: "2010"
description: "A running example for the whole narrative"
isbn: "978-0321712943"
url: "https://martinfowler.com/books/dsl.html"
tags:
  - "dsl"
  - "language design"
  - "patterns"
---

This chapter introduces DSLs through a concrete running example, then generalizes in the next chapter.

::: note
The state machine model is an example of a DSL.

That is to say, a DSL isn't necessarily text, it's a modeling "language" (by language, I mean notation, etc.). The text is usually a **serilization** of that language.
:::

## The Gothic Security System

Scenario: A company installs security systems in old castles. Each castle has hidden rooms that open through a specific sequence of actions (close a door, open a drawer, turn on a light, etc.).

The system has three parts:

- **Sensors** detect physical actions and send events (e.g. `D2OP` = "drawer 2 opened").
- **Devices (Actuators)** receive commands and do things (e.g. `D1UL` = "unlock door 1").
- **A controller** listens to events, tracks what has happened, and sends commands when the right sequence is done.

Each customer needs a different sequence. The controller is naturally modeled as a **state machine**.

## Miss Grant's Controller

Miss Grant's room unlocks when she **closes her bedroom door** and **opens a drawer** or **turns on a light** (in either order).

**Opening the door** at any point **resets the system back to idle**.

Basic jargons:

- **Sensors** detect user actions and send **events** (messages) (e.g. `doorClosed`, `drawerOpened`, `lightOn`) to the **controller**.
- The **controller** sends **commands** (messages) (e.g. `unlockPanel`, `lockPanel`, `unlockDoor`, `lockDoor`) to the **actuators**.
- The **controller** is always in one of the finite set of states. Entering a **state** can trigger **commands**.
- **Reset events** jump the system back to idle from any state. They are not strictly necessary but simplify the model.

The set of **states**:

- **idle**: The starting state.
  - On entry, runs commands `unlockDoor` and `lockPanel`.
- **active**:
  - Entered when `doorClosed` fires.
  - No commands are triggered.
- **waitingForDrawer**: Waiting for the drawer to be opened.
  - Entered from **active** on `lightOn` event.
- **waitingForLight**: Waiting for the light to be turned on.
  - Entered from active on `drawerOpened`.
- **unlockedPanel**:
  - Entered when both events have fired.
  - Runs `unlockPanel` and `lockDoor` on entry.

```mermaid
stateDiagram-v2
    [*] --> idle
    idle --> active : doorClosed

    active --> waitingForDrawer : lightOn
    active --> waitingForLight : drawerOpened

    waitingForDrawer --> unlockedPanel : drawerOpened
    waitingForLight --> unlockedPanel : lightOn

    unlockedPanel --> idle : panelClosed

    note right of idle
        Reset Events: doorOpened
        Commands: unlockDoor, lockPanel
    end note

    note right of unlockedPanel
        Commands: unlockPanel, lockDoor
    end note
```

## The State Machine Model

The **state machine** can be a good model for this case (both for communication and actually solving it).

Basically, the user will think about the controller in terms of: **events**, **states** and **transitions**. If these terms can be embedded in the code, then users can verify the business logic represented by the code (users and programmers share a language).

::: tip
This is the Domain-Driven Design (DDD) principle. The **domain model** is the shared conceptual vocabulary between domain experts (who describe how the security system should work) and programmers (who implement it). Both sides think in terms of states, events, transitions, and commands. The code structure mirrors the domain structure.
:::

The domain model for the state machine framework:

${fref("./_assets/01-an-introductory-example-1789797361.png")}

- A **state machine** has:
  - 1 **starting state**
  - A set of **reset events**, which will reset the state machine to the **starting state**.
- A **state** has:
  - A mapping from **event code** to a **transition**.
  - A set of **commands**.
- A **transition** specifies the **source** and **target** state along with the **event** that triggers it.
