---
uid Microsoft.Quantum.Math.LargestFixedPoint
title: LargestFixedPoint function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Math
qsharp.name: LargestFixedPoint
qsharp.summary: Returns the largest representable number for specific fixed point dimensions.
---

# LargestFixedPoint function

Namespace: [Microsoft.Quantum.Math](xref:Microsoft.Quantum.Math)

```qsharp
function LargestFixedPoint(integerBits : Int, fractionalBits : Int) : Double
```

## Summary
Returns the largest representable number for specific fixed point dimensions.

## Input
### integerBits
Number of integer bits (including the sign bit).
### fractionalBits
Number of fractional bits.

## Remark
The value can be computed as 2^(p-1) - 2^(-q), where p
is the number of integer bits and q is the number of fractional bits.
