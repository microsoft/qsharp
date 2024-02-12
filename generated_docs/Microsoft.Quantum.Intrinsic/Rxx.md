---
uid Microsoft.Quantum.Intrinsic.Rxx
title: Rxx operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Intrinsic
qsharp.name: Rxx
qsharp.summary: Applies the two qubit Ising _XX_ rotation gate.
---

# Rxx operation

Namespace: [Microsoft.Quantum.Intrinsic](xref:Microsoft.Quantum.Intrinsic)

```qsharp
operation Rxx(theta : Double, qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl
```

## Summary
Applies the two qubit Ising _XX_ rotation gate.

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
    R_{xx}(\theta) \mathrel{:=}
    \begin{bmatrix}
        \cos \theta & 0 & 0 & -i\sin \theta  \\\\
        0 & \cos \theta & -i\sin \theta & 0  \\\\
        0 & -i\sin \theta & \cos \theta & 0  \\\\
        -i\sin \theta & 0 & 0 & \cos \theta
    \end{bmatrix}.
\end{align}
$$
