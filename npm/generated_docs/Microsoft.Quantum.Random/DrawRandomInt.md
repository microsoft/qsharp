---
uid: Microsoft.Quantum.Random.DrawRandomInt
title: DrawRandomInt operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Random
qsharp.name: DrawRandomInt
qsharp.summary: Draws a random integer in a given inclusive range.
---

# DrawRandomInt operation

Namespace: [Microsoft.Quantum.Random](xref:Microsoft.Quantum.Random)

```qsharp
operation DrawRandomInt(min : Int, max : Int) : Int
```

## Summary
Draws a random integer in a given inclusive range.

## Input
### min
The smallest integer to be drawn.
### max
The largest integer to be drawn.

## Output
An integer in the inclusive range from `min` to `max` with uniform
probability.

## Remarks
Fails if `max < min`.

## Example
The following Q# snippet randomly rolls a six-sided die:
```qsharp
let roll = DrawRandomInt(1, 6);
```
