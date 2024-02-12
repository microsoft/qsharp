---
uid Microsoft.Quantum.Measurement.MResetZ
title: MResetZ operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Measurement
qsharp.name: MResetZ
qsharp.summary: Measures a single qubit in the Z basis,
and resets it to a fixed initial state
following the measurement.
---

# MResetZ operation

Namespace: [Microsoft.Quantum.Measurement](xref:Microsoft.Quantum.Measurement)

```qsharp
operation MResetZ(target : Qubit) : Result
```

## Summary
Measures a single qubit in the Z basis,
and resets it to a fixed initial state
following the measurement.

## Description
Performs a single-qubit measurement in the Z-basis,
and ensures that the qubit is returned to |0⟩
following the measurement.

## Input
### target
A single qubit to be measured.

## Output
The result of measuring `target` in the Pauli Z basis.
