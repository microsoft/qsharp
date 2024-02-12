---
uid: Microsoft.Quantum.Canon.ApplyXorInPlaceL
title: ApplyXorInPlaceL operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Canon
qsharp.name: ApplyXorInPlaceL
qsharp.summary: Applies a bitwise-XOR operation between a classical integer and an
integer represented by a register of qubits.
---

# ApplyXorInPlaceL operation

Namespace: [Microsoft.Quantum.Canon](xref:Microsoft.Quantum.Canon)

```qsharp
operation ApplyXorInPlaceL(value : BigInt, target : Qubit[]) : Unit is Adj + Ctl
```

## Summary
Applies a bitwise-XOR operation between a classical integer and an
integer represented by a register of qubits.

## Description
Applies `X` operations to qubits in a little-endian register based on
1 bits in an integer.

Let us denote `value` by a and let y be an unsigned integer encoded in `target`,
then `ApplyXorInPlace` performs an operation given by the following map:
|y⟩ ↦ |y ⊕ a⟩, where ⊕ is the bitwise exclusive OR operator.
