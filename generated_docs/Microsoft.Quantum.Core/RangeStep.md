---
uid Microsoft.Quantum.Core.RangeStep
title: RangeStep function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Core
qsharp.name: RangeStep
qsharp.summary: Returns the integer that specifies how the next value of a range is calculated.
---

# RangeStep function

Namespace: [Microsoft.Quantum.Core](xref:Microsoft.Quantum.Core)

```qsharp
function RangeStep(r : Range) : Int
```

## Summary
Returns the integer that specifies how the next value of a range is calculated.

## Input
### r
Input range.

## Output
The defined step value of the given range.

## Remarks
A range expression's first element is `start`,
its second element is `start+step`, third element is `start+step+step`, etc.,
until `end` is passed.
