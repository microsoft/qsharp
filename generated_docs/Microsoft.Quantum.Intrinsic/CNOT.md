---
uid Microsoft.Quantum.Intrinsic.CNOT
title: CNOT operation
ms.date: todo
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Intrinsic
qsharp.name: CNOT
qsharp.summary: Applies the controlled-NOT (CNOT) gate to a pair of qubits.
---

# CNOT operation

Namespace: [Microsoft.Quantum.Intrinsic](xref:Microsoft.Quantum.Intrinsic)

Applies the controlled-NOT (CNOT) gate to a pair of qubits.
```qsharp
operation CNOT(control : Qubit, target : Qubit) : Unit is Adj + Ctl
```

## Summary
Applies the controlled-NOT (CNOT) gate to a pair of qubits.

## Input
### control
Control qubit for the CNOT gate.
### target
Target qubit for the CNOT gate.

## Remarks
$$
\begin{align}
    \operatorname{CNOT} \mathrel{:=}
    \begin{bmatrix}
        1 & 0 & 0 & 0 \\\\
        0 & 1 & 0 & 0 \\\\
        0 & 0 & 0 & 1 \\\\
        0 & 0 & 1 & 0
    \end{bmatrix},
\end{align}
$$

where rows and columns are ordered as in the quantum concepts guide.

Equivalent to:
```qsharp
Controlled X([control], target);
```
