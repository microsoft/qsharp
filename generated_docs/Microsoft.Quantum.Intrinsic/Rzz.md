---
uid Microsoft.Quantum.Intrinsic.Rzz
title: Rzz operation
ms.date: 02/12/2024
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Intrinsic
qsharp.name: Rzz
qsharp.summary: Applies the two qubit Ising _ZZ_ rotation gate.
---

# Rzz operation

Namespace: [Microsoft.Quantum.Intrinsic](xref:Microsoft.Quantum.Intrinsic)

```qsharp
operation Rzz(theta : Double, qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl
```

## Summary
Applies the two qubit Ising _ZZ_ rotation gate.

## Input
### theta
The angle about which the qubits are rotated.
### qubit0
The first qubit input to the gate.
### qubit1
The second qubit input to the gate.

## Remarks
$$
\begin{align}
    R_{zz}(\theta) \mathrel{:=}
    \begin{bmatrix}
        e^{-i \theta / 2} & 0 & 0 & 0 \\\\
        0 & e^{i \theta / 2} & 0 & 0 \\\\
        0 & 0 & e^{i \theta / 2} & 0 \\\\
        0 & 0 & 0 & e^{-i \theta / 2}
    \end{bmatrix}.
\end{align}
$$
