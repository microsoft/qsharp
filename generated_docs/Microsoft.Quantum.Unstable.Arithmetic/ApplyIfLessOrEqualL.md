---
uid Microsoft.Quantum.Unstable.Arithmetic.ApplyIfLessOrEqualL
title: ApplyIfLessOrEqualL operation
ms.date: todo
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Unstable.Arithmetic
qsharp.name: ApplyIfLessOrEqualL
qsharp.summary: Computes `if (c <= x) { action(target) }`, that is, applies `action` to `target`
if a BigInt value `c` is less or equal to the little-endian qubit register `x`
---

# ApplyIfLessOrEqualL operation

Namespace: [Microsoft.Quantum.Unstable.Arithmetic](xref:Microsoft.Quantum.Unstable.Arithmetic)

```qsharp
operation ApplyIfLessOrEqualL<'T>(action : ('T => Unit is Param<1>), c : BigInt, x : Qubit[], target : 'T) : Unit is Adj + Ctl
```

## Summary
Computes `if (c <= x) { action(target) }`, that is, applies `action` to `target`
if a BigInt value `c` is less or equal to the little-endian qubit register `x`
