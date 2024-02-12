---
uid Microsoft.Quantum.Arrays.Windows
title: Windows function
ms.date: todo
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: Windows
qsharp.summary: Returns all consecutive subarrays of length `size`.
---

# Windows function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

Returns all consecutive subarrays of length `size`.
```qsharp
function Windows<'T>(size : Int, array : 'T[]) : 'T[][]
```

## Summary
Returns all consecutive subarrays of length `size`.

## Description
This function returns all `n - size + 1` subarrays of
length `size` in order, where `n` is the length of `array`.
The first subarrays are `array[0..size - 1], array[1..size], array[2..size + 1]`
until the last subarray `array[n - size..n - 1]`.

## Type Parameters
### 'T
The type of `array` elements.

## Input
### size
Length of the subarrays.

### array
An array of elements.

## Example
```qsharp
// same as [[1, 2, 3], [2, 3, 4], [3, 4, 5]]
let windows = Windows(3, [1, 2, 3, 4, 5]);
```

## Remarks
The size of the window must be a positive integer no greater than the size of the array
