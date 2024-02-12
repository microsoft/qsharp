---
uid Microsoft.Quantum.Math.SmallestFixedPoint
title: SmallestFixedPoint function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Math
qsharp.name: SmallestFixedPoint
qsharp.summary: Returns the smallest representable number for specific fixed point dimensions.
---

# SmallestFixedPoint function

Namespace: [Microsoft.Quantum.Math](xref:Microsoft.Quantum.Math)

```qsharp
function SmallestFixedPoint(integerBits : Int, fractionalBits : Int) : Double
```

## Summary
Returns the smallest representable number for specific fixed point dimensions.

## Input
### integerBits
Number of integer bits (including the sign bit).
### fractionalBits
Number of fractional bits.

## Remark
The value can be computed as -2^(p-1), where p is the number of integer bits.
