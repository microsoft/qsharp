---
uid Microsoft.Quantum.Measurement.MResetY
title: MResetY operation
ms.date: todo
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Measurement
qsharp.name: MResetY
qsharp.summary: Measures a single qubit in the Y basis,
and resets it to a fixed initial state
following the measurement.
---

# MResetY operation

Namespace: [Microsoft.Quantum.Measurement](xref:Microsoft.Quantum.Measurement)

Measures a single qubit in the Y basis,
and resets it to a fixed initial state
following the measurement.
```qsharp
operation MResetY(target : Qubit) : Result
```

## Summary
Measures a single qubit in the Y basis,
and resets it to a fixed initial state
following the measurement.

## Description
Performs a single-qubit measurement in the Y-basis,
and ensures that the qubit is returned to |0⟩
following the measurement.

## Input
### target
A single qubit to be measured.

## Output
The result of measuring `target` in the Pauli Y basis.
