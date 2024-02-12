---
uid Microsoft.Quantum.Unstable.Arithmetic.ApplyIfGreaterL
title: ApplyIfGreaterL operation
ms.date: todo
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Unstable.Arithmetic
qsharp.name: ApplyIfGreaterL
qsharp.summary: Computes `if (c > x) { action(target) }`, that is, applies `action` to `target`
if a BigInt value `c` is greater than the little-endian qubit register `x`
---

# ApplyIfGreaterL operation

Namespace: [Microsoft.Quantum.Unstable.Arithmetic](xref:Microsoft.Quantum.Unstable.Arithmetic)

Computes `if (c > x) { action(target) }`, that is, applies `action` to `target`
if a BigInt value `c` is greater than the little-endian qubit register `x`
```qsharp
operation ApplyIfGreaterL<'T>(action : ('T => Unit is Param<1>), c : BigInt, x : Qubit[], target : 'T) : Unit is Adj + Ctl
```

## Summary
Computes `if (c > x) { action(target) }`, that is, applies `action` to `target`
if a BigInt value `c` is greater than the little-endian qubit register `x`
