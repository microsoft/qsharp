---
uid Microsoft.Quantum.Unstable.Arithmetic.ApplyIfGreaterLE
title: ApplyIfGreaterLE operation
ms.date: 02/12/2024
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Unstable.Arithmetic
qsharp.name: ApplyIfGreaterLE
qsharp.summary: Computes `if x > y { action(target) }`, that is, applies `action` to `target`
if register `x` is greater than the register `y`.
Both qubit registers should be in a little-endian format.
---

# ApplyIfGreaterLE operation

Namespace: [Microsoft.Quantum.Unstable.Arithmetic](xref:Microsoft.Quantum.Unstable.Arithmetic)

```qsharp
operation ApplyIfGreaterLE<'T>(action : ('T => Unit is Param<1>), x : Qubit[], y : Qubit[], target : 'T) : Unit is Adj + Ctl
```

## Summary
Computes `if x > y { action(target) }`, that is, applies `action` to `target`
if register `x` is greater than the register `y`.
Both qubit registers should be in a little-endian format.
