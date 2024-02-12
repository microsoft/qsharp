---
uid Microsoft.Quantum.Unstable.Arithmetic.IncByLEUsingAddLE
title: IncByLEUsingAddLE operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Unstable.Arithmetic
qsharp.name: IncByLEUsingAddLE
qsharp.summary: Generic operation to turn two out-place adders into one in-place adder
---

# IncByLEUsingAddLE operation

Namespace: [Microsoft.Quantum.Unstable.Arithmetic](xref:Microsoft.Quantum.Unstable.Arithmetic)

```qsharp
operation IncByLEUsingAddLE(forwardAdder : ((Qubit[], Qubit[], Qubit[]) => Unit is Param<0>), backwardAdder : ((Qubit[], Qubit[], Qubit[]) => Unit is Param<1>), xs : Qubit[], ys : Qubit[]) : Unit is Adj + Ctl
```

## Summary
Generic operation to turn two out-place adders into one in-place adder

## Description
This implementation allows to specify two distinct adders for forward
and backward direction.  The forward adder is always applied in its
body variant, whereas the backward adder is always applied in its adjoint
variant.  Therefore, it's possible to, for example, use the ripple-carry
out-of-place adder in backwards direction to require no T gates.

The controlled variant is also optimized in a way that everything but
the adders is controlled,

## Reference
    - [arXiv:2012.01624](https://arxiv.org/abs/2012.01624)
      "Quantum block lookahead adders and the wait for magic states"
      by by Craig Gidney.
