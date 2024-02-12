---
uid Microsoft.Quantum.Unstable.Arithmetic.FourierTDIncByLE
title: FourierTDIncByLE operation
ms.date: 02/12/2024
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Unstable.Arithmetic
qsharp.name: FourierTDIncByLE
qsharp.summary: Increments a little-endian register ys by a little-endian register xs
using Quantum Fourier Transform.
---

# FourierTDIncByLE operation

Namespace: [Microsoft.Quantum.Unstable.Arithmetic](xref:Microsoft.Quantum.Unstable.Arithmetic)

```qsharp
operation FourierTDIncByLE(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl
```

## Summary
Increments a little-endian register ys by a little-endian register xs
using Quantum Fourier Transform.

## Description
Computes ys += xs modulo 2ⁿ, where xs and ys are little-endian registers,
and Length(xs) = Length(ys) = n.
This operation uses Quantum Fourier Transform.

## Reference
    - [arXiv:quant-ph/0008033](https://arxiv.org/abs/quant-ph/0008033)
     "Addition on a Quantum Computer" by Thomas G. Draper
