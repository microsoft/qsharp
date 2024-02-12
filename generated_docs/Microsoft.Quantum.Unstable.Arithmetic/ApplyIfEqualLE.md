---
uid Microsoft.Quantum.Unstable.Arithmetic.ApplyIfEqualLE
title: ApplyIfEqualLE operation
ms.date: 02/12/2024
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Unstable.Arithmetic
qsharp.name: ApplyIfEqualLE
qsharp.summary: Computes `if x == y { action(target) }`, that is, applies `action` to `target`
if register `x` is equal to the register `y`.
Both qubit registers should be in a little-endian format.
---

# ApplyIfEqualLE operation

Namespace: [Microsoft.Quantum.Unstable.Arithmetic](xref:Microsoft.Quantum.Unstable.Arithmetic)

```qsharp
operation ApplyIfEqualLE<'T>(action : ('T => Unit is Param<1>), x : Qubit[], y : Qubit[], target : 'T) : Unit is Adj + Ctl
```

## Summary
Computes `if x == y { action(target) }`, that is, applies `action` to `target`
if register `x` is equal to the register `y`.
Both qubit registers should be in a little-endian format.
