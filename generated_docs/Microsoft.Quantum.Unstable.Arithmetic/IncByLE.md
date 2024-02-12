---
uid Microsoft.Quantum.Unstable.Arithmetic.IncByLE
title: IncByLE operation
ms.date: todo
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Unstable.Arithmetic
qsharp.name: IncByLE
qsharp.summary: Increments a little-endian register ys by a little-endian register xs
---

# IncByLE operation

Namespace: [Microsoft.Quantum.Unstable.Arithmetic](xref:Microsoft.Quantum.Unstable.Arithmetic)

Increments a little-endian register ys by a little-endian register xs
```qsharp
operation IncByLE(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl
```

## Summary
Increments a little-endian register ys by a little-endian register xs

## Description
Computes ys += xs modulo 2ⁿ, where xs and ys are little-endian registers,
and Length(xs) ≤ Length(ys) = n.
NOTE: Use operations like RippleCarryCGIncByLE directly if
the choice of implementation is important.
