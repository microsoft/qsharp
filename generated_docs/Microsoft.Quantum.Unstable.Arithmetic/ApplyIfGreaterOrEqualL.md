---
uid Microsoft.Quantum.Unstable.Arithmetic.ApplyIfGreaterOrEqualL
title: ApplyIfGreaterOrEqualL operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Unstable.Arithmetic
qsharp.name: ApplyIfGreaterOrEqualL
qsharp.summary: Computes `if (c >= x) { action(target) }`, that is, applies `action` to `target`
if a BigInt value `c` is greater or equal to the little-endian qubit register `x`
---

# ApplyIfGreaterOrEqualL operation

Namespace: [Microsoft.Quantum.Unstable.Arithmetic](xref:Microsoft.Quantum.Unstable.Arithmetic)

```qsharp
operation ApplyIfGreaterOrEqualL<'T>(action : ('T => Unit is Param<1>), c : BigInt, x : Qubit[], target : 'T) : Unit is Adj + Ctl
```

## Summary
Computes `if (c >= x) { action(target) }`, that is, applies `action` to `target`
if a BigInt value `c` is greater or equal to the little-endian qubit register `x`
