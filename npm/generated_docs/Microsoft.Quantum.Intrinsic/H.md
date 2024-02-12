---
uid: Microsoft.Quantum.Intrinsic.H
title: H operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Intrinsic
qsharp.name: H
qsharp.summary: Applies the Hadamard transformation to a single qubit.
---

# H operation

Namespace: [Microsoft.Quantum.Intrinsic](xref:Microsoft.Quantum.Intrinsic)

```qsharp
operation H(qubit : Qubit) : Unit is Adj + Ctl
```

## Summary
Applies the Hadamard transformation to a single qubit.

## Input
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    e^{i \theta [P_0 \otimes P_1 \cdots P_{N-1}]},
\end{align}
$$
where $P_i$ is the $i$th element of `paulis`, and where
$N = $`Length(paulis)`.
