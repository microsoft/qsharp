---
uid Microsoft.Quantum.Intrinsic.Y
title: Y operation
ms.date: 02/12/2024
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Intrinsic
qsharp.name: Y
qsharp.summary: Applies the Pauli _Y_ gate.
---

# Y operation

Namespace: [Microsoft.Quantum.Intrinsic](xref:Microsoft.Quantum.Intrinsic)

```qsharp
operation Y(qubit : Qubit) : Unit is Adj + Ctl
```

## Summary
Applies the Pauli _Y_ gate.

## Input
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    \sigma_y \mathrel{:=}
    \begin{bmatrix}
        0 & -i \\\\
        i & 0
    \end{bmatrix}.
\end{align}
$$
