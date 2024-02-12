---
uid Microsoft.Quantum.Canon.CY
title: CY operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Canon
qsharp.name: CY
qsharp.summary: Applies the controlled-Y (CY) gate to a pair of qubits.
---

# CY operation

Namespace: [Microsoft.Quantum.Canon](xref:Microsoft.Quantum.Canon)

```qsharp
operation CY(control : Qubit, target : Qubit) : Unit is Adj + Ctl
```

## Summary
Applies the controlled-Y (CY) gate to a pair of qubits.

## Input
### control
Control qubit for the CY gate.
### target
Target qubit for the CY gate.

## Remarks
This operation can be simulated by the unitary matrix
$$
\begin{align}
    1 & 0 & 0 & 0 \\\\
    0 & 1 & 0 & 0 \\\\
    0 & 0 & 0 & -i \\\\
    0 & 0 & i & 0
\end{align},
$$
where rows and columns are organized as in the quantum concepts guide.

Equivalent to:
```qsharp
Controlled Y([control], target);
```
