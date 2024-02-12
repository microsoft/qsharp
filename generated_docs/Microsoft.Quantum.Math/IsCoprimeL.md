---
uid Microsoft.Quantum.Math.IsCoprimeL
title: IsCoprimeL function
ms.date: todo
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Math
qsharp.name: IsCoprimeL
qsharp.summary: Returns if two integers are co-prime.
---

# IsCoprimeL function

Namespace: [Microsoft.Quantum.Math](xref:Microsoft.Quantum.Math)

Returns if two integers are co-prime.
```qsharp
function IsCoprimeL(a : BigInt, b : BigInt) : Bool
```

## Summary
Returns if two integers are co-prime.

## Description
Returns true if a and b are co-prime and false otherwise.

## Input
### a
the first number of which co-primality is being tested
### b
the second number of which co-primality is being tested

## Output
True, if a and b are co-prime (e.g. their greatest common divisor is 1),
and false otherwise
