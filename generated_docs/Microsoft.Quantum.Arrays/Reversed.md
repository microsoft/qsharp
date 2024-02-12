---
uid Microsoft.Quantum.Arrays.Reversed
title: Reversed function
ms.date: todo
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: Reversed
qsharp.summary: Create an array that contains the same elements as an input array but in reversed
order.
---

# Reversed function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

Create an array that contains the same elements as an input array but in reversed
order.
```qsharp
function Reversed<'T>(array : 'T[]) : 'T[]
```

## Summary
Create an array that contains the same elements as an input array but in reversed
order.

## Type Parameters
### 'T
The type of the array elements.

## Input
### array
An array whose elements are to be copied in reversed order.

## Output
An array containing the elements `array[Length(array) - 1]` .. `array[0]`.
