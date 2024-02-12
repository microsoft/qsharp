---
uid Microsoft.Quantum.Arrays.SequenceL
title: SequenceL function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: SequenceL
qsharp.summary: Get an array of integers in a given interval.
---

# SequenceL function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

```qsharp
function SequenceL(from : BigInt, to : BigInt) : BigInt[]
```

## Summary
Get an array of integers in a given interval.

## Input
### from
An inclusive start index of the interval.
### to
An inclusive end index of the interval that is not smaller than `from`.

## Output
An array containing the sequence of numbers `from`, `from + 1`, ...,
`to`.

## Remarks
The difference between `from` and `to` must fit into an `Int` value.

## Example
```qsharp
let arr1 = SequenceL(0L, 3L); // [0L, 1L, 2L, 3L]
let arr2 = SequenceL(23L, 29L); // [23L, 24L, 25L, 26L, 27L, 28L, 29L]
let arr3 = SequenceL(-5L, -2L); // [-5L, -4L, -3L, -2L]
```
