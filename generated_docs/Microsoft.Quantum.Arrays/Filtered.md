---
uid Microsoft.Quantum.Arrays.Filtered
title: Filtered function
ms.date: todo
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: Filtered
qsharp.summary: Given an array and a predicate that is defined
for the elements of the array, returns an array that consists of
those elements that satisfy the predicate.
---

# Filtered function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

Given an array and a predicate that is defined
for the elements of the array, returns an array that consists of
those elements that satisfy the predicate.
```qsharp
function Filtered<'T>(predicate : ('T -> Bool), array : 'T[]) : 'T[]
```

## Summary
Given an array and a predicate that is defined
for the elements of the array, returns an array that consists of
those elements that satisfy the predicate.

## Type Parameters
### 'T
The type of `array` elements.

## Input
### predicate
A function from `'T` to Boolean that is used to filter elements.
### array
An array of elements over `'T`.

## Output
An array `'T[]` of elements that satisfy the predicate.

## Example
The following code creates an array that contains only even numbers.
```qsharp
Filtered(x -> x % 2 == 0, [0, 1, 2, 3, 4])
```
