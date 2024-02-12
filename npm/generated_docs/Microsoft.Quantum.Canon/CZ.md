---
uid: Microsoft.Quantum.Canon.CZ
title: CZ operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Canon
qsharp.name: CZ
qsharp.summary: Applies the controlled-Z (CZ) gate to a pair of qubits.
---

# CZ operation

Namespace: [Microsoft.Quantum.Canon](xref:Microsoft.Quantum.Canon)

```qsharp
operation CZ(control : Qubit, target : Qubit) : Unit is Adj + Ctl
```

## Summary
Applies the controlled-Z (CZ) gate to a pair of qubits.

## Input
### control
Control qubit for the CZ gate.
### target
Target qubit for the CZ gate.

## Remarks
This operation can be simulated by the unitary matrix
$$
\begin{align}
    1 & 0 & 0 & 0 \\\\
    0 & 1 & 0 & 0 \\\\
    0 & 0 & 1 & 0 \\\\
    0 & 0 & 0 & -1
\end{align},
$$
where rows and columns are organized as in the quantum concepts guide.

Equivalent to:
```qsharp
Controlled Z([control], target);
```
