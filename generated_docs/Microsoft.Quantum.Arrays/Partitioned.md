---
uid Microsoft.Quantum.Arrays.Partitioned
title: Partitioned function
ms.date: todo
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: Partitioned
qsharp.summary: Splits an array into multiple parts.
---

# Partitioned function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

```qsharp
function Partitioned<'T>(partitionSizes : Int[], array : 'T[]) : 'T[][]
```

## Summary
Splits an array into multiple parts.

## Input
### partitionSizes
Number of elements in each splitted part of array.
### array
Input array to be split.

## Output
Multiple arrays where the first array is the first `partitionSizes[0]` of `array`
and the second array are the next `partitionSizes[1]` of `array` etc. The last array
will contain all remaining elements. If the array is split exactly, the
last array will be the empty array, indicating there are no remaining elements.
In other words, `Tail(Partitioned(...))` will always return the remaining
elements, while `Most(Partitioned(...))` will always return the complete
partitions of the array.

## Example
```qsharp
// The following returns [[2, 3], [5], [7]];
let split = Partitioned([2, 1], [2, 3, 5, 7]);
// The following returns [[2, 3], [5, 7], []];
let split = Partitioned([2, 2], [2, 3, 5, 7]);
```
