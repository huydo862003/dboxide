---
_type: Book
_label: "Implementing an Internal DSL"
author: "Martin Fowler"
date: "2010"
description: "Techniques for building internal DSLs: fluent interfaces, function patterns, closures"
isbn: "978-0321712943"
url: "https://martinfowler.com/books/dsl.html"
tags:
  - "dsl"
  - "internal dsl"
  - "patterns"
---

Internal DSLs have a language-like flow but are constrained by the host language's syntax.

## Fluent vs Command-Query APIs

- A **command-query API** has methods that are self-sufficient and make sense in isolation (like buttons on a machine).
  - It follows **command-query separation**: queries return values without side effects, commands change state without returning values.
- A **fluent interface** has methods that only make sense within a larger expression. Naming is sentence-oriented, not button-oriented.
  - Method chaining typically breaks command-query separation because each method returns `this` to continue the chain. This is acceptable because fluent interfaces follow different rules.

Do not mix both styles on the same class. Keep the semantic model's command-query API clean and put fluent methods on a separate **Expression Builder** layer.

::: details Example: separating the builder from the model

The semantic model has a clean command-query API:

```java
// Semantic Model with command-query API
class State {
    void addTransition(Event trigger, State target) { ... }
    void addAction(Command action) { ... }
    boolean hasTransition(String eventCode) { ... }
}
```

The Expression Builder wraps it with a fluent interface:

```java
// Expression Builder with fluent API
class StateBuilder {
    private State state;

    StateBuilder transition(Event trigger) {
        this.pendingTrigger = trigger;
        return this;  // breaks command-query separation
    }

    StateBuilder to(State target) {
        state.addTransition(pendingTrigger, target);
        return this;
    }

    StateBuilder actions(Command... cmds) {
        for (Command c : cmds) state.addAction(c);
        return this;
    }
}
```

The DSL script talks to the builder. The builder talks to the model. They never share a class.
:::

## Using Functions

Three patterns for combining functions into a DSL:

- **Method Chaining**: `.processor().cores(2).speed(2500)`.
  - Each method returns `this` or a related builder to continue the chain.
  - Pros:
    - Optional elements are easy: just skip the call.
    - The method name identifies which element you are setting.
  - Cons:
    - Breaks command-query separation.
    - Finishing the chain needs an explicit `end()` or similar terminator.
- **Function Sequence**: `processor(); cores(2); speed(2500);`.
  - Requires global functions and context variables to track which object is being built.
  - Pros:
    - Simple to implement.
    - Calls execute in the order they are written.
  - Cons:
    - Global functions pollute the namespace and global state makes the code harder to follow.
- **Nested Function**: `computer(processor(cores(2), speed(2500)), disk(size(150)))`.
  - Arguments are nested inside higher-level function calls, reflecting the tree structure directly.
  - Pros:
    - The hierarchy is explicit in the code and no context variables are needed.
    - The parent function receives fully formed children.
  - Cons:
    - Evaluation order is inside-out (`third(second(first()))` reads backwards).
    - Positional arguments lack names. Languages with keyword arguments or Literal Maps fix this.
- **Object Scoping**: place the DSL script in a subclass of the Expression Builder so that all function calls resolve as instance methods.
  - Pros:
    - All functions are scoped to the builder, so they do not pollute the global namespace.
    - Parsing data lives on the builder instance as instance fields, eliminating global state.
    - Users can extend the DSL by subclassing and adding their own methods.
  - Cons:
    - Forces users to write a subclass for each DSL script.
  - This is Fowler's default recommendation for Function Sequence.

::: details Object Scoping in the state machine example

This only really works in Java or C++ as `this` is implicit.

```java
abstract class StateMachineBuilder {
    private State currentState;
    protected void state(String name) { currentState = new State(name); }
    protected void transition(String event, String target) { ... }
    protected void actions(String... cmds) { ... }
    protected abstract void define();
}

class MissGrantsController extends StateMachineBuilder {
    protected void define() {
        state("idle");
            actions("unlockDoor", "lockPanel");
            transition("doorClosed", "active");
        state("active");
            transition("drawerOpened", "waitingForLight");
    }
}
```

The calls inside `define()` look like bare functions but resolve as instance methods inherited from `StateMachineBuilder`.
:::

## Literal Collections

- A **Literal List** captures a homogeneous list of elements.

```ruby
actions :unlockDoor, :lockPanel
```

- A **Literal Map** uses key-value pairs for named properties. Each value is identified by its key, avoiding positional argument ambiguity.

```ruby
transitions :doorClosed => :active,
            :lightOn    => :waitingForDrawer
```

::: details Choosing a function pattern based on grammar structure

In practice, you combine all three patterns. A common hybrid uses Function Sequence for top-level items, Nested Function for each item's structure, and Method Chaining for optional properties.

| Structure | BNF | Consider |
|---|---|---|
| Mandatory list | `parent ::= first second third` | Nested Function |
| Optional list | `parent ::= first maybeSecond?` | Method Chaining, Literal Map |
| Homogeneous bag | `parent ::= child*` | Literal List, Function Sequence |
| Heterogeneous bag | `parent ::= (this \| that)*` | Method Chaining |
| Set | n/a | Literal Map |

:::

## Closures

Closures (lambdas, blocks, anonymous functions) are used as **Nested Closures** in DSLs.

```ruby
ComputerBuilder.build do |c|
  c.processor do |p|
    p.cores 2
    p.speed 2.2
  end
  c.disk do |d|
    d.size 150
  end
end
```

Three properties that make them useful for DSLs:

- **Inline nesting**: closures capture hierarchy in the language itself.
  - Unlike Nested Function, closures can contain arbitrary statements, including Function Sequences inside them.
- **Deferred evaluation**: you control when closures execute.
  - A DSL can store host code blocks in the semantic model and run them later. This is key for adaptive models.

```csharp
Validate("Annual Income is present")
    .With(p => p.AnnualIncome != null);
```

- **Limited-scope variables**: block parameters (`|c|`, `|p|`, `|d|`) make it clear which object each closure is manipulating.
  - This also eliminates the need for global functions or Object Scoping.

::: note
Closure syntax varies wildly across languages.
- Ruby and Smalltalk have clean syntax, making Nested Closures practical.
- Java lacks closures entirely. C# supports them but with noisier syntax.
- Lisp has first-class closures but awkward syntax, which it works around with macros.
:::

## Parse Tree Manipulation

Instead of evaluating a host-language expression, you can capture its parse tree as data and walk it to generate different output.

For example, `aPerson.Age > 18` in C# can be captured as a tree:

```
BinaryExpression (>)
├── MemberAccess: aPerson.Age
└── Constant: 18
```

You can then walk this tree to generate SQL, XPath, or keep it as an in-memory query. This is how C# LINQ works: a single C# expression translates to different backends depending on the target.

- Useful when you want to write expressions in the host language but execute them in a different environment.
- Requires language-level support for capturing expression trees. Most languages compile expressions immediately with no way to get the tree.
- Rarely needed, but very powerful when it applies.

::: note
Two languages that support this:

- **C#** has `Expression<Func<T>>`. Writing `Expression<Func<Person, bool>> expr = p => p.Age > 18;` gives you a tree object instead of a compiled function. LINQ to SQL walks this tree and generates SQL from it.
- **Lisp** gets this for free through **homoiconicity**: code is data (nested lists), so you can manipulate it directly with macros. No special API needed.
:::

## Annotations

Annotations can be thought of as extending the host language with new domain-specific keywords.

```java
@ValidRange(lower = 1, upper = 1000, units = Units.LB)
private Quantity weight;
```

- Common uses: validation rules, ORM mappings (`@Entity`, `@Table`), access control, serialization hints.
- They read more clearly than putting the same logic in a setter or a separate config file.
- They specify the rule declaratively, making it easy to check at compile time, runtime, or even by a GUI widget builder.
- Only suited for fragmentary DSLs because they always need a host code element to attach to.

## Literal Extension

Literal Extension adds methods to existing types, letting you start a method chain from a literal value.

```ruby
5.days.ago
3.hours.from_now
```

- The chain starts from an integer literal, which normally has no domain-specific methods. Ruby's open classes and C#'s extension methods make this possible.
- This gives the DSL a very natural reading order, but global extensions pollute the namespace. Use sparingly.

## Reducing Syntactic Noise

- **Textual Polishing** does text substitution before parsing, converting `3 hours ago` to `3.hours.ago`. It gets convoluted quickly, and a real external DSL is usually a better choice.
- **Syntax coloring** can de-emphasize noisy host syntax in the editor so domain experts only see the DSL parts. You can even color host syntax the same as the background to make it disappear.

## Dynamic Reception

In dynamic languages like Ruby and Smalltalk, you can override `method_missing` to handle undefined method calls at runtime. This moves information from method arguments into method names:

- `people.find_by_firstname("martin")` instead of `people.find("firstname", "martin")`.

Dynamic Reception works best for simple, repetitive patterns like building queries by property name. Do not use it for complex structures because method name parsing gets unwieldy.

## Type Checking for DSL Symbols

DSL symbols are usually strings, so a misspelled `"waitingForLihgt"` silently creates a broken model.

A **Class Symbol Table** turns strings into typed fields:

```java
class BasicStateMachine extends StateMachineBuilder {
    States idle, active, waitingForLight, unlockedPanel;
    Events doorClosed, drawerOpened, lightOn;
    Commands unlockPanel, lockDoor;

    protected void define() {
        waitingForLight
            .transition(lightOn).to(unlockedPanel);
    }
}
```

- The compiler catches typos and the IDE autocompletes symbol names.
- The framework initializes each field from its field name using reflection.
