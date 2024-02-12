---
uid Microsoft.Quantum.Arrays.Chunks
title: Chunks function
ms.date: todo
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: Chunks
qsharp.summary: Splits an array into multiple parts of equal length.
---

# Chunks function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

```qsharp
function Chunks<'T>(chunkSize : Int, array : 'T[]) : 'T[][]
```

## Summary
Splits an array into multiple parts of equal length.

## Input
### chunkSize
The length of each chunk. Must be positive.
### array
The array to be split in chunks.

## Output
A array containing each chunk of the original array.

## Remarks
Note that the last element of the output may be shorter
than `chunkSize` if `Length(array)` is not divisible by `chunkSize`.
