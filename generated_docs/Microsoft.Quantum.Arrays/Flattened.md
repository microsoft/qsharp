---
uid Microsoft.Quantum.Arrays.Flattened
title: Flattened function
ms.date: todo
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: Flattened
qsharp.summary: Given an array of arrays, returns the concatenation of all arrays.
---

# Flattened function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

```qsharp
function Flattened<'T>(arrays : 'T[][]) : 'T[]
```

## Summary
Given an array of arrays, returns the concatenation of all arrays.

## Type Parameters
### 'T
The type of `array` elements.

## Input
### arrays
Array of arrays.

## Output
Concatenation of all arrays.

## Example
```qsharp
let flattened = Flattened([[1, 2], [3], [4, 5, 6]]);
// flattened = [1, 2, 3, 4, 5, 6]
```
