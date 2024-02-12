---
uid Microsoft.Quantum.Intrinsic.SWAP
title: SWAP operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Intrinsic
qsharp.name: SWAP
qsharp.summary: Applies the SWAP gate to a pair of qubits.
---

# SWAP operation

Namespace: [Microsoft.Quantum.Intrinsic](xref:Microsoft.Quantum.Intrinsic)

```qsharp
operation SWAP(qubit1 : Qubit, qubit2 : Qubit) : Unit is Adj + Ctl
```

## Summary
Applies the SWAP gate to a pair of qubits.

## Input
### qubit1
First qubit to be swapped.
### qubit2
Second qubit to be swapped.

## Remarks
$$
\begin{align}
    \operatorname{SWAP} \mathrel{:=}
    \begin{bmatrix}
        1 & 0 & 0 & 0 \\\\
        0 & 0 & 1 & 0 \\\\
        0 & 1 & 0 & 0 \\\\
        0 & 0 & 0 & 1
    \end{bmatrix},
\end{align}
$$

where rows and columns are ordered as in the quantum concepts guide.

Equivalent to:
```qsharp
CNOT(qubit1, qubit2);
CNOT(qubit2, qubit1);
CNOT(qubit1, qubit2);
```
