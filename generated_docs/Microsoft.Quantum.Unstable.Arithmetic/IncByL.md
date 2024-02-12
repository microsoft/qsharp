---
uid Microsoft.Quantum.Unstable.Arithmetic.IncByL
title: IncByL operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Unstable.Arithmetic
qsharp.name: IncByL
qsharp.summary: Increments a little-endian register ys by a BigInt number c
---

# IncByL operation

Namespace: [Microsoft.Quantum.Unstable.Arithmetic](xref:Microsoft.Quantum.Unstable.Arithmetic)

```qsharp
operation IncByL(c : BigInt, ys : Qubit[]) : Unit is Adj + Ctl
```

## Summary
Increments a little-endian register ys by a BigInt number c

## Description
Computes ys += c modulo 2ⁿ, where ys is a little-endian register,
Length(ys) = n > 0, c is a BigInt number, 0 ≤ c < 2ⁿ.
NOTE: Use IncByLUsingIncByLE directly if the choice of implementation
is important.
