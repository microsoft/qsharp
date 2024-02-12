---
uid: Microsoft.Quantum.Math.PNorm
title: PNorm function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Math
qsharp.name: PNorm
qsharp.summary: Returns the `L(p)` norm of a vector of `Double`s.

That is, given an array x of type `Double[]`, this returns the p-norm
|x̄|ₚ= (∑(xᵢ)ᵖ)¹ᐟᵖ.
---

# PNorm function

Namespace: [Microsoft.Quantum.Math](xref:Microsoft.Quantum.Math)

```qsharp
function PNorm(p : Double, array : Double[]) : Double
```

## Summary
Returns the `L(p)` norm of a vector of `Double`s.

That is, given an array x of type `Double[]`, this returns the p-norm
|x̄|ₚ= (∑(xᵢ)ᵖ)¹ᐟᵖ.

## Input
### p
The exponent p in the p-norm.

## Output
The p-norm |x̄|ₚ.
