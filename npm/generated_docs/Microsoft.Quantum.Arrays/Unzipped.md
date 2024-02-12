---
uid: Microsoft.Quantum.Arrays.Unzipped
title: Unzipped function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: Unzipped
qsharp.summary: Given an array of 2-tuples, returns a tuple of two arrays, each containing
the elements of the tuples of the input array.
---

# Unzipped function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

```qsharp
function Unzipped<'T, 'U>(array : ('T, 'U)[]) : ('T[], 'U[])
```

## Summary
Given an array of 2-tuples, returns a tuple of two arrays, each containing
the elements of the tuples of the input array.

## Type Parameters
### 'T
The type of the first element in each tuple.
### 'U
The type of the second element in each tuple.

## Input
### array
An array containing 2-tuples.

## Output
Two arrays, the first one containing all first elements of the input
tuples, the second one containing all second elements of the input tuples.

## Example
```qsharp
// split is same as ([5, 4, 3, 2, 1], [true, false, true, true, false])
let split = Unzipped([(5, true), (4, false), (3, true), (2, true), (1, false)]);
```

## See Also
- Microsoft.Quantum.Arrays.Zipped
