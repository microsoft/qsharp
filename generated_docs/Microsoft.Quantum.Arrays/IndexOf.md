---
uid Microsoft.Quantum.Arrays.IndexOf
title: IndexOf function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: IndexOf
qsharp.summary: Returns the first index of the first element in an array that satisfies
a given predicate. If no such element exists, returns -1.
---

# IndexOf function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

```qsharp
function IndexOf<'T>(predicate : ('T -> Bool), array : 'T[]) : Int
```

## Summary
Returns the first index of the first element in an array that satisfies
a given predicate. If no such element exists, returns -1.

## Input
### predicate
A predicate function acting on elements of the array.
### array
An array to be searched using the given predicate.

## Output
Either the smallest index of an element for which `predicate(array[index])` is true,
or -1 if no such element exists.

## Example
The following code gets the index of the first even number in the input array.
```qsharp
let indexOfFirstEven = IndexOf(x -> x % 2 == 0, [1, 3, 17, 2, 21]);
// `indexOfFirstEven` is 3.
```
