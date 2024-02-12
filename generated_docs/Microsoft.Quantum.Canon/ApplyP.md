---
uid Microsoft.Quantum.Canon.ApplyP
title: ApplyP operation
ms.date: todo
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Canon
qsharp.name: ApplyP
qsharp.summary: Given a single-qubit Pauli operator, applies the corresponding operation
to a single qubit.
---

# ApplyP operation

Namespace: [Microsoft.Quantum.Canon](xref:Microsoft.Quantum.Canon)

```qsharp
operation ApplyP(pauli : Pauli, target : Qubit) : Unit is Adj + Ctl
```

## Summary
Given a single-qubit Pauli operator, applies the corresponding operation
to a single qubit.

## Input
### pauli
The Pauli operator to be applied.
### target
The qubit to which `pauli` is to be applied as an operation.

## Example
The following are equivalent:
```qsharp
ApplyP(PauliX, q);
```
and
```qsharp
X(q);
```
