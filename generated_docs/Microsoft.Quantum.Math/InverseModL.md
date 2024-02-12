---
uid Microsoft.Quantum.Math.InverseModL
title: InverseModL function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Math
qsharp.name: InverseModL
qsharp.summary: Returns the multiplicative inverse of a modular integer.
---

# InverseModL function

Namespace: [Microsoft.Quantum.Math](xref:Microsoft.Quantum.Math)

```qsharp
function InverseModL(a : BigInt, modulus : BigInt) : BigInt
```

## Summary
Returns the multiplicative inverse of a modular integer.

## Description
This will calculate the multiplicative inverse of a
modular integer `b` such that `a • b = 1 (mod modulus)`.
