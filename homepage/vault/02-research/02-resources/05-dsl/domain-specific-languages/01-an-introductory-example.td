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
