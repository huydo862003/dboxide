---
_type: Book
_label: "Code Generation"
author: "Martin Fowler"
date: "2010"
description: "When and how to generate code from a DSL's semantic model"
isbn: "978-0321712943"
url: "https://martinfowler.com/books/dsl.html"
tags:
  - "dsl"
  - "code generation"
---

::: tip
- Code generation is optional. Most of the time, executing the semantic model directly is simpler.
- Use it when the target environment cannot run the model (embedded hardware, SQL, a different language).
- The DSL processor (parser + semantic model + generator) and the target environment are two separate worlds.
:::

## What to Generate

Two styles, illustrated with a simple on/off state machine:

**Model Ignorant Generation**: the state machine logic is baked into control flow. No representation of the model exists in the generated code.

```java
public void handle(Event event) {
    switch (currentState) {
        case ON:  if (event == DOWN) currentState = OFF; break;
        case OFF: if (event == UP)   currentState = ON;  break;
    }
}
```

**Model Aware Generation**: the generated code includes a representation of the semantic model as data. A generic framework interprets that data at runtime.

```java
modelMachine = new ModelMachine(OFF);
modelMachine.defineTransition(OFF, UP, ON);
modelMachine.defineTransition(ON, DOWN, OFF);
```

- Model Aware Generation produces simpler generated code because you only generate the configuration, not the behavior. The framework can be built and tested independently in the target environment.
- Model Ignorant Generation is needed when the target cannot represent a model as data, or when the processing overhead of interpreting a model is too great (common in embedded systems).
- You can go further with Model Aware Generation and generate a plain text table that the target loads at startup, allowing runtime behavior changes without recompilation.

## How to Generate

- **Transformer Generation**: write code that walks the semantic model and emits output statements. You control the traversal.
- **Templated Generation**: write a template file in the target language with placeholders that call into the semantic model. The template engine fills them in.

::: details When to use which
- Templated Generation works well when the output is mostly static with a few dynamic parts. You can look at the template and see what the output will look like. Pairs naturally with Model Ignorant Generation.
- Transformer Generation works better when the output structure differs significantly from the template structure. Preferred most of the time.
- In practice, you mix both. Transformer Generation code often uses string formatting for small chunks, which is essentially miniature Templated Generation.
:::

::: note
With Templated Generation, keep template logic minimal. Put all complexity in an **Embedment Helper** class that the template calls with simple method calls. This keeps the template readable and lets you edit the helper in a real IDE with syntax highlighting and type checking.
:::

## Mixing Generated and Handwritten Code

Two rules:

- Never modify generated code by hand. The DSL is the authoritative source. Manual edits are lost on regeneration.
- Keep generated and handwritten code in separate files.

::: details Strategies for mixing in OO languages
- **Partial classes** (C#): split generated and handwritten code into separate files for the same class.
- **Generation Gap**: generate a superclass, handwrite a subclass. The subclass can augment and override generated behavior. This is the most portable solution.
- Do not check generated code into version control. It can be regenerated at will during the build process.
:::

## Generating Readable Code

- Generated code should be well-structured with clear variable names. You will need to read it when debugging.
- Duplication matters less than in handwritten code because you never modify it directly.
- Comments in generated code are useful because they can reference structures in the semantic model, helping you trace back to the DSL.

## Preparse Code Generation

Code generation can also run *before* the DSL is written, not just after. You generate artifacts that help the DSL script author, such as enum types listing valid symbols from an external data source. This gives the DSL script compile-time checking and IDE autocompletion for externally-defined data.
