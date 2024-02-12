---
uid: Microsoft.Quantum.Math.FactorialI
title: FactorialI function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Math
qsharp.name: FactorialI
qsharp.summary: Returns the factorial of a given number.
---

# FactorialI function

Namespace: [Microsoft.Quantum.Math](xref:Microsoft.Quantum.Math)

```qsharp
function FactorialI(n : Int) : Int
```

## Summary
Returns the factorial of a given number.

## Description
Returns the factorial of a given nonnegative integer n, where 0 ≤ n ≤ 20.

## Input
### n
The number to take the factorial of.

## Output
The factorial of `n`.

## Remarks
For inputs greater than 20, please use `Microsoft.Quantum.Math.FactorialL`.

## See Also
- Microsoft.Quantum.Math.FactorialL
- Microsoft.Quantum.Math.ApproximateFactorial
