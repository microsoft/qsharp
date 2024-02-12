---
uid Microsoft.Quantum.Core.RangeStart
title: RangeStart function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Core
qsharp.name: RangeStart
qsharp.summary: Returns the defined start value of the given range.
---

# RangeStart function

Namespace: [Microsoft.Quantum.Core](xref:Microsoft.Quantum.Core)

```qsharp
function RangeStart(r : Range) : Int
```

## Summary
Returns the defined start value of the given range.

## Input
### r
Input range.

## Output
The defined start value of the given range.

## Remarks
A range expression's first element is `start`,
its second element is `start+step`, third element is `start+step+step`, etc.,
until `end` is passed.

Note that the defined start value of a range is the same as the first element of the sequence,
unless the range specifies an empty sequence (for example, 2 .. 1).
