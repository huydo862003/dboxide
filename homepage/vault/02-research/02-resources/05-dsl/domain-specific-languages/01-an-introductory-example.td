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
Recap:

- Semantic model: The general mechanism/logic that can be instantiated with various configurations.
- DSL: The language that describes the configurations for the semantic model.

At runtime, we have the DSL engine containing the semantic model's logic & we parse the static DSL to populate the semantic model with a configuration.
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

## Configuration: Programming Miss Grant's Controller

::: tip
The domain model above is **common code**: how state machines work. Basically, it describes **how the model works** regardless of **configurations**.

In this section, we configure the model to work with an instance of the problem it's designed to work with. This configuration is **variable code**: what Miss Grant's controller does. Basically, it applies a configuration to the model.
:::

::: note
It feels like the separation between the **model** and the **configuration** is in actuality the separation between **behavior**/**logic**/**code** and **data**.
:::

### Imperative: Java

So in Java, the controller logic can be implemented like this:

```java
/* init events */
Event doorClosed   = new Event("doorClosed", "D1CL");
Event drawerOpened = new Event("drawerOpened", "D2OP");
// ...

/* init commands */
Command unlockPanelCmd = new Command("unlockPanel", "PNUL");
Command lockPanelCmd   = new Command("lockPanel", "PNLK");
// ...

/* init states & add transitions/actions */
State idle               = new State("idle");
State unlockedPanelState = new State("unlockedPanel");
// ...

idle.addTransition(doorClosed, activeState);
idle.addAction(unlockDoorCmd);
idle.addAction(lockPanelCmd);
// ...

StateMachine machine = new StateMachine(idle);
machine.addResetEvents(doorOpened);
```

### Declarative: XML Configuration

The same configuration can be expressed as XML:

```xml
<stateMachine start="idle">
  <event name="doorClosed" code="D1CL"/>
  <event name="drawerOpened" code="D2OP"/>
  <!-- ... -->

  <command name="unlockPanel" code="PNUL"/>
  <command name="lockPanel" code="PNLK"/>
  <!-- ... -->

  <state name="idle">
    <transition event="doorClosed" target="active"/>
    <action command="unlockDoor"/>
    <action command="lockPanel"/>
  </state>

  <state name="active">
    <transition event="drawerOpened" target="waitingForLight"/>
    <transition event="lightOn" target="waitingForDrawer"/>
  </state>

  <!-- ... -->

  <state name="unlockedPanel">
    <action command="unlockPanel"/>
    <action command="lockDoor"/>
    <transition event="panelClosed" target="idle"/>
  </state>

  <resetEvent name="doorOpened"/>
</stateMachine>
```

Advantages over the Java wiring:

- Separation between **model** and **configuration**: No recompilation needed. Ship the state machine JAR once, swap XML files per customer.
  > Remark: This further highlights the separation between **code** and **data**
- **Declarative**. No variable shuffling. Transitions and actions are expressed as sub-elements of their parent state, not as method calls on object references.
- **Limited by design**. The XML can only express configuration, not arbitrary logic. This constraint reduces the surface area for mistakes.

The tradeoff is that syntax errors move from compile time to runtime. XML schema validation helps, but does not fully replace type checking.

::: note
This is why so many Java/C# frameworks ended up configured with XML files. The configuration is data, and XML is a data format.
:::

But XML is noisy. Can we do better?

### Custom Syntax (External DSL)

```
events
  doorClosed   D1CL
  drawerOpened D2OP
  lightOn      L1ON
  doorOpened   D1OP
  panelClosed  PNCL
end

resetEvents
  doorOpened
end

commands
  unlockPanel PNUL
  lockPanel   PNLK
  lockDoor    D1LK
  unlockDoor  D1UL
end

state idle
  actions {unlockDoor lockPanel}
  doorClosed => active
end

state active
  drawerOpened => waitingForLight
  lightOn      => waitingForDrawer
end

state waitingForLight
  lightOn => unlockedPanel
end

state waitingForDrawer
  drawerOpened => unlockedPanel
end

state unlockedPanel
  actions {unlockPanel lockDoor}
  panelClosed => idle
end
```

A custom syntax Fowler made up for this example.

- Terser than XML: No quoting, no angle brackets, no closing tags.
- Not Turing-complete: Can only configure this particular kind of state machine. This limitation is a feature: easy to edit, process, and verify.
- Readable by non-programmers: The people who set up the security system can read this and spot errors, even if they cannot read the Java code.
- Requires a custom parser: Unlike XML, there is no off-the-shelf tooling to parse it.

::: note
This is an **external DSL**: a language with its own custom syntax, separate from the host programming language.
:::

### Ruby (Internal DSL)

```ruby
event :doorClosed,   "D1CL"
event :drawerOpened, "D2OP"
event :lightOn,      "L1ON"
event :doorOpened,   "D1OP"
event :panelClosed,  "PNCL"

command :unlockPanel, "PNUL"
command :lockPanel,   "PNLK"
command :lockDoor,    "D1LK"
command :unlockDoor,  "D1UL"

resetEvents :doorOpened

state :idle do
  actions :unlockDoor, :lockPanel
  transitions :doorClosed => :active
end

state :active do
  transitions :drawerOpened => :waitingForLight,
              :lightOn      => :waitingForDrawer
end

state :waitingForLight do
  transitions :lightOn => :unlockedPanel
end

state :waitingForDrawer do
  transitions :drawerOpened => :unlockedPanel
end

state :unlockedPanel do
  actions :unlockPanel, :lockDoor
  transitions :panelClosed => :idle
end
```

This is valid Ruby. It uses a subset of Ruby's capabilities to look like a custom language. Ruby's syntactic flexibility (symbols, blocks, hash rockets) makes this possible.

::: note
This is an **internal DSL**: a DSL embedded within a host language's syntax. Lisp programmers have done this for decades. The term **fluent interface** is sometimes used as a synonym.
:::

### Java (Internal DSL)

The original Java configuration code is NOT a DSL. It is API wiring. But Java can also host an internal DSL:

```java
public class BasicStateMachine extends StateMachineBuilder {
    Events   doorClosed, drawerOpened, lightOn, panelClosed;
    Commands unlockPanel, lockPanel, lockDoor, unlockDoor;
    States   idle, active, waitingForLight, waitingForDrawer, unlockedPanel;
    ResetEvents doorOpened;

    protected void defineStateMachine() {
        doorClosed .code("D1CL");
        drawerOpened.code("D2OP");
        // ...

        idle
            .actions(unlockDoor, lockPanel)
            .transition(doorClosed).to(active);

        active
            .transition(drawerOpened).to(waitingForLight)
            .transition(lightOn).to(waitingForDrawer);

        // ...

        unlockedPanel
            .actions(unlockPanel, lockDoor)
            .transition(panelClosed).to(idle);
    }
}
```

It uses unusual formatting and fluent method chaining, but it is valid Java. **The declarative flow is what makes it a DSL rather than just an API**.

::: tip
What separates an internal DSL from a normal API? Fowler calls it "a rather fuzzy notion of a language-like flow." A regular API is a **command-query API**: you call methods to get things done. A DSL has a **fluent interface**: you compose declarations that read like a language.
:::

### Recap: External vs Internal DSLs

All four representations (Java API, XML, custom syntax, Ruby) configure the same domain model. The difference is in how the configuration is expressed.

- An **external DSL** has its own syntax, separate from the host language. The custom syntax and the XML are both external DSLs.
- An **internal DSL** is a stylized use of a general-purpose language for a domain-specific purpose. The Ruby and Java fluent versions are internal DSLs.

::: note
Most XML configuration files are essentially external DSLs wrapped in an XML carrier syntax. The XML is just **a serialization format for the real language (that describes the configuration for a semantic model) underneath**.
:::

## Languages and Semantic Model

**The role of a DSL is to populate the semantic model**.

```mermaid
flowchart LR
    subgraph Input
        DSL["events\n  doorClosed D1CL\nend\n\nstate idle\n  doorClosed => active\nend"]
    end

    DSL -->|populates| SM

    subgraph SM [Semantic Model]
        S1[":State\nname = idle"]
        S2[":State\nname = active"]
        T[":Transition"]
        E[":Event\nname = doorClosed\ncode = D1CL"]
        T --> S1
        T --> S2
        T --> E
    end
```

### Semantic Model vs Domain Model

From the DSL's point of view, Fowler calls the **domain model** a **semantic model**.

- Syntax is what the DSL looks like (captured by the grammar).
- Semantics is what it means (captured by the model).

For practical purposes, a semantic model is _very close to a domain model_. The difference is perspective: domain model emphasizes the business domain, semantic model emphasizes the DSL's meaning.

> Essentially, the semantic model decides what a DSL can describe.

::: tip
Fowler advocates that a semantic model is a vital part of a well-designed DSL. It provides a clear separation of concerns:

- You can reason about the model without worrying about language issues.
- You can test the model independently by populating it with a command-query API.
- You can evolve the model and the DSL independently.
- All four representations (Java API, XML, custom syntax, Ruby) built on the same model and created the same object configuration.
:::

> It's more like this: When you design a DSL, it's a good architecture to separate the concerns between "parsing"/"translating" to the "model" and the "model" internal logic.

The semantic model is the more important part:

- The model is the engine that provides behavior. The DSL is a thin facade that populates it.
- The model brings runtime configurability. The DSL makes that configurability readable.

### Adaptive Model

A state machine is an **adaptive model**: a model whose population effectively acts as the program for the system. Changing behavior means altering the objects and their wiring, not changing code. **This blurs the distinction between code and data**.

Adaptive models are powerful but hard to use because there is no visible code that defines the behavior.

A DSL solves this by providing an explicit, readable representation of the configuration. The DSL gives people the sensation of programming the model.

### Alternative Computational Models

State machines, production rule systems, and dependency networks are **alternative computational models**.

> Turing machine is a computation model that closely models our computer. But sometimes, it can be too low-level.

- An adaptive model is a good way to implement alternative computational models.
- A DSL is a good way to make them programmable.

This is often called **declarative programming**.

### Model-First vs DSL-First

Fowler describes a **model-first process**: build the model, then layer a DSL over it. But the reverse also works. You can design the DSL with domain experts first, then build the model and DSL simultaneously. Either way, the model and DSL end up as separate concerns.

## Interpretation vs Code Generation

Two approaches to processing a DSL:

- **Interpretation**: Parse the DSL, populate the semantic model, execute the model directly.
- **Code generation** (compilation): Parse the DSL, populate the semantic model, generate code from it. The generated code runs separately.

Code generation is useful when:

- The target platform differs from the DSL tooling platform (generate C, SQL, COBOL, MathCAD).
- You want to avoid shipping the parser and model to production.

Code generation is NOT useful when:

- People treat it as the primary goal. It is just an implementation mechanism.
- People skip the semantic model and generate code directly from the parse tree. This couples parsing to output and makes everything harder to change.

::: tip
A semantic model decouples parsing, semantics, and output. You can change the DSL syntax without touching the code generator. You can generate multiple outputs from the same model. You can use both interpretation and code generation off the same model.
:::

## Language Workbenches

- Tools that let you define a DSL AND its editing environment (highlighting, completion, diagnostics) together. They solve the main disadvantage of external DSLs: limited tooling.
- Can go beyond text: diagrammatic languages, tables, projectional editing (edit the AST directly).
- Examples: JetBrains MPS, Xtext/Langium, Spoofax.
