---
_type: Concept
_label: "Attribute-Value Matrix (AVM)"
status: 'learning'
tags:
  - 'type system'
  - 'linguistics'
---

A bracket notation for writing feature structures.

$$
\left[\begin{array}{ll}\textsc{name} & \text{string} \\ \textsc{population} & \text{int}\end{array}\right]
$$

Values can be nested (the value of an attribute is itself a feature structure):

$$
\left[\begin{array}{ll}\textsc{person} & \left[\begin{array}{ll}\textsc{name} & \text{string} \\ \textsc{age} & \text{int}\end{array}\right]\end{array}\right]
$$

Reentrancy (shared substructure) is marked by tagging two values with the same index, meaning two paths point to the same node.
