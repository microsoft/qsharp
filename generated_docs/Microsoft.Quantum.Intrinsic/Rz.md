---
uid Microsoft.Quantum.Intrinsic.Rz
title: Rz operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Intrinsic
qsharp.name: Rz
qsharp.summary: Applies a rotation about the _z_-axis by a given angle.
---

# Rz operation

Namespace: [Microsoft.Quantum.Intrinsic](xref:Microsoft.Quantum.Intrinsic)

```qsharp
operation Rz(theta : Double, qubit : Qubit) : Unit is Adj + Ctl
```

## Summary
Applies a rotation about the _z_-axis by a given angle.

## Input
### theta
Angle about which the qubit is to be rotated.
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    R_z(\theta) \mathrel{:=}
    e^{-i \theta \sigma_z / 2} =
    \begin{bmatrix}
        e^{-i \theta / 2} & 0 \\\\
        0 & e^{i \theta / 2}
    \end{bmatrix}.
\end{align}
$$

Equivalent to:
```qsharp
R(PauliZ, theta, qubit);
```
