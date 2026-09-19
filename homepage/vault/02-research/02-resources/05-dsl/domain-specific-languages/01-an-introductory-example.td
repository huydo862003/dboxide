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

::: details AbstractEvent
The base class for both commands and events. Each has a human-readable name and a machine-readable code.

```java
class AbstractEvent {
    private String name, code;

    public AbstractEvent(String name, String code) {
        this.name = name;
        this.code = code;
    }

    public String getCode() { return code; }
    public String getName() { return name; }
}

class Command extends AbstractEvent { }
class Event extends AbstractEvent { }
```
:::

::: details State
A state has a name, a set of actions (commands to execute on entry), and a map from event codes to transitions. The map acts as a lookup: given an event code, which transition do we take?

```java
class State {
    private String name;
    private List<Command> actions = new ArrayList<>();
    private Map<String, Transition> transitions = new HashMap<>();

    public void addTransition(Event event, State targetState) {
        transitions.put(event.getCode(), new Transition(this, event, targetState));
    }

    public boolean hasTransition(String eventCode) {
        return transitions.containsKey(eventCode);
    }

    public State targetState(String eventCode) {
        return transitions.get(eventCode).getTarget();
    }

    public void executeActions(CommandChannel channel) {
        for (Command c : actions) channel.send(c.getCode());
    }
}
```
:::

::: details Transition
A transition connects a source state to a target state, triggered by an event.

```java
class Transition {
    private final State source, target;
    private final Event trigger;

    public Transition(State source, Event trigger, State target) {
        this.source = source;
        this.target = target;
        this.trigger = trigger;
    }

    public State getSource() { return source; }
    public State getTarget() { return target; }
    public Event getTrigger() { return trigger; }
    public String getEventCode() { return trigger.getCode(); }
}
```
:::

::: details StateMachine*
Holds the start state and a list of reset events. Reset events jump the machine back to the start state from any state.

The sole purpose of this class is for the initial configuration of the controller, which is a bit misleading

```java
class StateMachine {
    private State start;
    private List<Event> resetEvents = new ArrayList<>();

    public StateMachine(State start) {
        this.start = start;
    }

    public void addResetEvents(Event... events) {
        for (Event e : events) resetEvents.add(e);
    }

    public boolean isResetEvent(String eventCode) {
        return resetEvents.stream()
            .anyMatch(e -> e.getCode().equals(eventCode));
    }

    public State getStart() { return start; }
}
```
:::

::: details Controller
The runtime that processes incoming events against the state machine. It tracks the current state and dispatches commands.

```java
class Controller {
    private State currentState;
    private StateMachine machine;
    private CommandChannel commandsChannel;

    public void handle(String eventCode) {
        if (currentState.hasTransition(eventCode))
            transitionTo(currentState.targetState(eventCode));
        else if (machine.isResetEvent(eventCode))
            transitionTo(machine.getStart());
        // ignore unknown events
    }

    private void transitionTo(State target) {
        currentState = target;
        currentState.executeActions(commandsChannel);
    }
}
```
:::
