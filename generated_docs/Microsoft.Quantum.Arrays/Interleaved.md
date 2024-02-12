---
uid Microsoft.Quantum.Arrays.Interleaved
title: Interleaved function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: Interleaved
qsharp.summary: Interleaves two arrays of (almost) same size.
---

# Interleaved function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

```qsharp
function Interleaved<'T>(first : 'T[], second : 'T[]) : 'T[]
```

## Summary
Interleaves two arrays of (almost) same size.

## Description
This function returns the interleaving of two arrays, starting
with the first element from the first array, then the first
element from the second array, and so on.

The first array must either be
of the same length as the second one, or can have one more element.

## Type Parameters
### 'T
The type of each element of `first` and `second`.

## Input
### first
The first array to be interleaved.

### second
The second array to be interleaved.

## Output
Interleaved array

## Example
```qsharp
// same as interleaved = [1, -1, 2, -2, 3, -3]
let interleaved = Interleaved([1, 2, 3], [-1, -2, -3])
```
