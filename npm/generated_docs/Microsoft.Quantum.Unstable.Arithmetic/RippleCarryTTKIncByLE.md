---
uid: Microsoft.Quantum.Unstable.Arithmetic.RippleCarryTTKIncByLE
title: RippleCarryTTKIncByLE operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Unstable.Arithmetic
qsharp.name: RippleCarryTTKIncByLE
qsharp.summary: Reversible, in-place ripple-carry addition of two integers.
---

# RippleCarryTTKIncByLE operation

Namespace: [Microsoft.Quantum.Unstable.Arithmetic](xref:Microsoft.Quantum.Unstable.Arithmetic)

```qsharp
operation RippleCarryTTKIncByLE(xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl
```

## Summary
Reversible, in-place ripple-carry addition of two integers.

## Description
Computes ys += xs modulo 2ⁿ, where xs and ys are little-endian registers,
and Length(xs) ≤ Length(ys) = n.
This operation uses the ripple-carry algorithm.
Note that if Length(ys) >= Length(xs)+2, xs is padded with 0-initialized
qubits to match ys's length. The operation doesn't use any auxilliary
qubits otherwise.

## References
    - [arXiv:0910.2530](https://arxiv.org/abs/0910.2530)
      "Quantum Addition Circuits and Unbounded Fan-Out"
      by Yasuhiro Takahashi, Seiichiro Tani, Noboru Kunihiro

