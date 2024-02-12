---
uid Microsoft.Quantum.Convert.ResultArrayAsInt
title: ResultArrayAsInt function
ms.date: 02/12/2024
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Convert
qsharp.name: ResultArrayAsInt
qsharp.summary: Produces a non-negative integer from a string of Results in little-endian format.
---

# ResultArrayAsInt function

Namespace: [Microsoft.Quantum.Convert](xref:Microsoft.Quantum.Convert)

```qsharp
function ResultArrayAsInt(results : Result[]) : Int
```

## Summary
Produces a non-negative integer from a string of Results in little-endian format.

## Input
### results
Results in binary representation of number.

## Output
A non-negative integer

## Example
```qsharp
// The following returns 1
let int1 = ResultArrayAsInt([One,Zero])
```
