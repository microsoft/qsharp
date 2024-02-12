---
uid Microsoft.Quantum.Core.Repeated
title: Repeated function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Core
qsharp.name: Repeated
qsharp.summary: Creates an array of given length with all elements equal to given value.
---

# Repeated function

Namespace: [Microsoft.Quantum.Core](xref:Microsoft.Quantum.Core)

```qsharp
function Repeated<'T>(value : 'T, length : Int) : 'T[]
```

## Summary
Creates an array of given length with all elements equal to given value.

## Input
### value
The value of each element of the new array.
### length
Length of the new array.

## Output
A new array of length `length`, such that every element is `value`.

## Example
The following code creates an array of 3 Boolean values, each equal to `true`:
```qsharp
let array = Repeated(true, 3);
```
