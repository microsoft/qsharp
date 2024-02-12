---
uid Microsoft.Quantum.Intrinsic.T
title: T operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Intrinsic
qsharp.name: T
qsharp.summary: Applies the π/8 gate to a single qubit.
---

# T operation

Namespace: [Microsoft.Quantum.Intrinsic](xref:Microsoft.Quantum.Intrinsic)

```qsharp
operation T(qubit : Qubit) : Unit is Adj + Ctl
```

## Summary
Applies the π/8 gate to a single qubit.

## Input
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    T \mathrel{:=}
    \begin{bmatrix}
        1 & 0 \\\\
        0 & e^{i \pi / 4}
    \end{bmatrix}.
\end{align}
$$
