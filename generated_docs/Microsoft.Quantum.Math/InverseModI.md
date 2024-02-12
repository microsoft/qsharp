---
uid Microsoft.Quantum.Math.InverseModI
title: InverseModI function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Math
qsharp.name: InverseModI
qsharp.summary: Returns the multiplicative inverse of a modular integer.
---

# InverseModI function

Namespace: [Microsoft.Quantum.Math](xref:Microsoft.Quantum.Math)

```qsharp
function InverseModI(a : Int, modulus : Int) : Int
```

## Summary
Returns the multiplicative inverse of a modular integer.

## Description
This will calculate the multiplicative inverse of a
modular integer `b` such that `a • b = 1 (mod modulus)`.
