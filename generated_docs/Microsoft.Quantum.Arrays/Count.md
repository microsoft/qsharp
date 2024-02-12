---
uid Microsoft.Quantum.Arrays.Count
title: Count function
ms.date: 02/12/2024
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: Count
qsharp.summary: Given an array and a predicate that is defined
for the elements of the array, returns the number of elements
an array that consists of those elements that satisfy the predicate.
---

# Count function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

```qsharp
function Count<'T>(predicate : ('T -> Bool), array : 'T[]) : Int
```

## Summary
Given an array and a predicate that is defined
for the elements of the array, returns the number of elements
an array that consists of those elements that satisfy the predicate.

## Type Parameters
### 'T
The type of `array` elements.

## Input
### predicate
A function from `'T` to Boolean that is used to filter elements.
### array
An array of elements over `'T`.

## Output
The number of elements in `array` that satisfy the predicate.

## Example
```qsharp
 let evensCount = Count(x -> x % 2 == 0, [1, 3, 6, 7, 9]);
// evensCount is 1.
```
