---
uid: Microsoft.Quantum.Math.ExtendedGreatestCommonDivisorL
title: ExtendedGreatestCommonDivisorL function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Math
qsharp.name: ExtendedGreatestCommonDivisorL
qsharp.summary: Returns a tuple (u,v) such that u*a+v*b=GCD(a,b)
Note: GCD is always positive except that GCD(0,0)=0.
---

# ExtendedGreatestCommonDivisorL function

Namespace: [Microsoft.Quantum.Math](xref:Microsoft.Quantum.Math)

```qsharp
function ExtendedGreatestCommonDivisorL(a : BigInt, b : BigInt) : (BigInt, BigInt)
```

## Summary
Returns a tuple (u,v) such that u*a+v*b=GCD(a,b)
Note: GCD is always positive except that GCD(0,0)=0.
