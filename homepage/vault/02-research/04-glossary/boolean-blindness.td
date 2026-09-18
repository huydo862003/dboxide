---
_type: Concept
_label: "Boolean Blindness"
status: 'mastered'
tags:
  - 'type system'
---

A situation where validation returns a simple boolean (`true`/`false`), causing the system to lose the proof of validity after the check. Instead of returning `bool`, prefer parsing into types that guarantee validity at the type level, preserving the validation evidence throughout the program.

Actually, when writing this, I figured this is related to proof theory's distinction between Admissibility and Derivability (I learned about these in PFPL - see my [type-theory](https://github.com/hdnax/type-theory) repo). Returning `bool` is like admissibility (asserting "this is valid" without proof), while returning a validated type is like derivability (carrying constructive evidence). When you return `bool`, you lose the derivation - just like admissibility is a weaker assertion than derivability because it lacks the constructive proof tree.
