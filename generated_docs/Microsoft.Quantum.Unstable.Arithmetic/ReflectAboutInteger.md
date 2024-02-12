---
uid Microsoft.Quantum.Unstable.Arithmetic.ReflectAboutInteger
title: ReflectAboutInteger operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Unstable.Arithmetic
qsharp.name: ReflectAboutInteger
qsharp.summary: Reflects a quantum register about a given classical integer.
---

# ReflectAboutInteger operation

Namespace: [Microsoft.Quantum.Unstable.Arithmetic](xref:Microsoft.Quantum.Unstable.Arithmetic)

```qsharp
operation ReflectAboutInteger(index : Int, reg : Qubit[]) : Unit is Adj + Ctl
```

## Summary
Reflects a quantum register about a given classical integer.

## Description
Given a quantum register initially in the state ∑ᵢ(αᵢ|i⟩),
where each |i⟩ is a basis state representing an integer i,
reflects the state of the register about the basis state |j⟩
for a given integer j: ∑ᵢ(-1)^(δᵢⱼ)(αᵢ|i⟩)

## Input
### index
The classical integer j indexing the basis state about which to reflect.
### reg
Little-endian quantum register to reflect.

## Remarks
This operation is implemented in-place, without explicit allocation of
additional auxiliary qubits.
