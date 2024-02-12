---
uid Microsoft.Quantum.Intrinsic.R1Frac
title: R1Frac operation
ms.date: todo
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Intrinsic
qsharp.name: R1Frac
qsharp.summary: Applies a rotation about the |1⟩ state by an angle specified
as a dyadic fraction.

WARNING:
This operation uses the **opposite** sign convention from
Microsoft.Quantum.Intrinsic.R, and does not include the
factor of 1/2 included by Microsoft.Quantum.Intrinsic.R1.
---

# R1Frac operation

Namespace: [Microsoft.Quantum.Intrinsic](xref:Microsoft.Quantum.Intrinsic)

Applies a rotation about the |1⟩ state by an angle specified
as a dyadic fraction.

WARNING:
This operation uses the **opposite** sign convention from
Microsoft.Quantum.Intrinsic.R, and does not include the
factor of 1/2 included by Microsoft.Quantum.Intrinsic.R1.
```qsharp
operation R1Frac(numerator : Int, power : Int, qubit : Qubit) : Unit is Adj + Ctl
```

## Summary
Applies a rotation about the |1⟩ state by an angle specified
as a dyadic fraction.

WARNING:
This operation uses the **opposite** sign convention from
Microsoft.Quantum.Intrinsic.R, and does not include the
factor of 1/2 included by Microsoft.Quantum.Intrinsic.R1.

## Input
### numerator
Numerator in the dyadic fraction representation of the angle
by which the qubit is to be rotated. This angle is expressed in radians.
### power
Power of two specifying the denominator of the angle by which
the qubit is to be rotated. This angle is expressed in radians.
### qubit
Qubit to which the gate should be applied.

## Remarks
$$
\begin{align}
    R_1(n, k) \mathrel{:=}
    \operatorname{diag}(1, e^{i \pi k / 2^n}).
\end{align}
$$

Equivalent to:
```qsharp
RFrac(PauliZ, -numerator, denominator + 1, qubit);
RFrac(PauliI, numerator, denominator + 1, qubit);
```
