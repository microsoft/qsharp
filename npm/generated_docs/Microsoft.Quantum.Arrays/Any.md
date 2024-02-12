---
uid: Microsoft.Quantum.Arrays.Any
title: Any function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: Any
qsharp.summary: Given an array and a predicate that is defined
for the elements of the array, checks if at least one element of
the array satisfies the predicate.
---

# Any function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

```qsharp
function Any<'T>(predicate : ('T -> Bool), array : 'T[]) : Bool
```

## Summary
Given an array and a predicate that is defined
for the elements of the array, checks if at least one element of
the array satisfies the predicate.

## Type Parameters
### 'T
The type of `array` elements.

## Input
### predicate
A function from `'T` to `Bool` that is used to check elements.
### array
An array of elements over `'T`.

## Output
A `Bool` value of the OR function of the predicate applied to all elements.

## Example
```qsharp
let anyEven = Any(x -> x % 2 == 0, [1, 3, 6, 7, 9]);
```
