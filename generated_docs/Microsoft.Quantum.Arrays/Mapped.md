---
uid Microsoft.Quantum.Arrays.Mapped
title: Mapped function
ms.date: todo
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: Mapped
qsharp.summary: Given an array and a function that is defined
for the elements of the array, returns a new array that consists
of the images of the original array under the function.
---

# Mapped function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

Given an array and a function that is defined
for the elements of the array, returns a new array that consists
of the images of the original array under the function.
```qsharp
function Mapped<'T, 'U>(mapper : ('T -> 'U), array : 'T[]) : 'U[]
```

## Summary
Given an array and a function that is defined
for the elements of the array, returns a new array that consists
of the images of the original array under the function.

## Type Parameters
### 'T
The type of `array` elements.
### 'U
The result type of the `mapper` function.

## Input
### mapper
A function from `'T` to `'U` that is used to map elements.
### array
An array of elements over `'T`.

## Output
An array `'U[]` of elements that are mapped by the `mapper` function.

## See Also
- Microsoft.Quantum.Arrays.ForEach
