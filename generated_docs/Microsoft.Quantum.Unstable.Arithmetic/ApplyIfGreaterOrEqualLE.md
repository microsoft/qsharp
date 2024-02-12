---
uid Microsoft.Quantum.Unstable.Arithmetic.ApplyIfGreaterOrEqualLE
title: ApplyIfGreaterOrEqualLE operation
ms.date: todo
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Unstable.Arithmetic
qsharp.name: ApplyIfGreaterOrEqualLE
qsharp.summary: Computes `if x >= y { action(target) }`, that is, applies `action` to `target`
if register `x` is greater or equal to the register `y`.
Both qubit registers should be in a little-endian format.
---

# ApplyIfGreaterOrEqualLE operation

Namespace: [Microsoft.Quantum.Unstable.Arithmetic](xref:Microsoft.Quantum.Unstable.Arithmetic)

Computes `if x >= y { action(target) }`, that is, applies `action` to `target`
if register `x` is greater or equal to the register `y`.
Both qubit registers should be in a little-endian format.
```qsharp
operation ApplyIfGreaterOrEqualLE<'T>(action : ('T => Unit is Param<1>), x : Qubit[], y : Qubit[], target : 'T) : Unit is Adj + Ctl
```

## Summary
Computes `if x >= y { action(target) }`, that is, applies `action` to `target`
if register `x` is greater or equal to the register `y`.
Both qubit registers should be in a little-endian format.
