---
uid Microsoft.Quantum.Arrays.MappedOverRange
title: MappedOverRange function
ms.date: 02/12/2024
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: MappedOverRange
qsharp.summary: Given a range and a function that takes an integer as input,
returns a new array that consists
of the images of the range values under the function.
---

# MappedOverRange function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

```qsharp
function MappedOverRange<'T>(mapper : (Int -> 'T), range : Range) : 'T[]
```

## Summary
Given a range and a function that takes an integer as input,
returns a new array that consists
of the images of the range values under the function.

## Type Parameters
### 'T
The result type of the `mapper` function.

## Input
### mapper
A function from `Int` to `'T` that is used to map range values.
### range
A range of integers.

## Output
An array `'T[]` of elements that are mapped by the `mapper` function.

## Example
This example adds 1 to a range of even numbers:
```qsharp
let numbers = MappedOverRange(x -> x + 1, 0..2..10);
// numbers = [1, 3, 5, 7, 9, 11]
```

## See Also
- Microsoft.Quantum.Arrays.Mapped
