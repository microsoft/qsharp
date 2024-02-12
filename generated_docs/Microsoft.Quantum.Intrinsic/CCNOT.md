---
uid Microsoft.Quantum.Intrinsic.CCNOT
title: CCNOT operation
ms.date: 02/12/2024
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Intrinsic
qsharp.name: CCNOT
qsharp.summary: Applies the doubly controlled–NOT (CCNOT) gate to three qubits.
---

# CCNOT operation

Namespace: [Microsoft.Quantum.Intrinsic](xref:Microsoft.Quantum.Intrinsic)

```qsharp
operation CCNOT(control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Adj + Ctl
```

## Summary
Applies the doubly controlled–NOT (CCNOT) gate to three qubits.

## Input
### control1
First control qubit for the CCNOT gate.
### control2
Second control qubit for the CCNOT gate.
### target
Target qubit for the CCNOT gate.

## Remarks
Equivalent to:
```qsharp
Controlled X([control1, control2], target);
```
