---
uid: Microsoft.Quantum.Arrays.IsSquareArray
title: IsSquareArray function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: IsSquareArray
qsharp.summary: Returns whether a 2-dimensional array has a square shape
---

# IsSquareArray function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

```qsharp
function IsSquareArray<'T>(array : 'T[][]) : Bool
```

## Summary
Returns whether a 2-dimensional array has a square shape

## Type Parameters
### 'T
The type of each element of `array`.

## Input
### array
A 2-dimensional array of elements.

## Example
```qsharp
IsSquareArray([[1, 2], [3, 4]]);         // true
IsSquareArray([[1, 2, 3], [4, 5, 6]]);   // false
IsSquareArray([[1, 2], [3, 4], [5, 6]]); // false
```

## Output
`true` if the array is square, `false` otherwise.

## See Also
- Microsoft.Quantum.Arrays.IsRectangularArray
