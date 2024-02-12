---
uid Microsoft.Quantum.Unstable.Arithmetic.MAJ
title: MAJ operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Unstable.Arithmetic
qsharp.name: MAJ
qsharp.summary: This applies the in-place majority operation to 3 qubits.
---

# MAJ operation

Namespace: [Microsoft.Quantum.Unstable.Arithmetic](xref:Microsoft.Quantum.Unstable.Arithmetic)

```qsharp
operation MAJ(x : Qubit, y : Qubit, z : Qubit) : Unit is Adj + Ctl
```

## Summary
This applies the in-place majority operation to 3 qubits.

## Description
Assuming the state of the input qubits are |x⟩, |y⟩ and |z⟩, then
this operation performs the following transformation:
|x⟩|y⟩|z⟩ ↦ |x ⊕ z⟩|y ⊕ z⟩MAJ(x, y, z).

## Input
### x
The first input qubit.
### y
The second input qubit.
### z
A qubit onto which the majority function will be applied.
