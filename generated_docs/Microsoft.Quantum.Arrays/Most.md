---
uid Microsoft.Quantum.Arrays.Most
title: Most function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: Most
qsharp.summary: Creates an array that is equal to an input array except that the last array
element is dropped.
---

# Most function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

```qsharp
function Most<'T>(array : 'T[]) : 'T[]
```

## Summary
Creates an array that is equal to an input array except that the last array
element is dropped.

## Type Parameters
### 'T
The type of the array elements.

## Input
### array
An array whose first to second-to-last elements are to form the output array.

## Output
An array containing the elements `array[0..Length(array) - 2]`.
