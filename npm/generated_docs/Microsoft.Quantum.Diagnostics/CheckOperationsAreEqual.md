---
uid: Microsoft.Quantum.Diagnostics.CheckOperationsAreEqual
title: CheckOperationsAreEqual operation
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Diagnostics
qsharp.name: CheckOperationsAreEqual
qsharp.summary: Given two operations, checks that they act identically for all input states.
---

# CheckOperationsAreEqual operation

Namespace: [Microsoft.Quantum.Diagnostics](xref:Microsoft.Quantum.Diagnostics)

```qsharp
operation CheckOperationsAreEqual(nQubits : Int, actual : (Qubit[] => Unit is Param<0>), expected : (Qubit[] => Unit is Param<1>)) : Bool
```

## Summary
Given two operations, checks that they act identically for all input states.

## Description
This check is implemented by using the Choi–Jamiołkowski isomorphism to reduce
this check to a check on two entangled registers.
Thus, this operation needs only a single call to each operation being tested,
but requires twice as many qubits to be allocated.
This check can be used to ensure, for instance, that an optimized version of an
operation acts identically to its naïve implementation, or that an operation
which acts on a range of non-quantum inputs agrees with known cases.

## Remarks
This operation requires that the operation modeling the expected behavior is
adjointable, so that the inverse can be performed on the target register alone.
Formally, one can specify a transpose operation, which relaxes this requirement,
but the transpose operation is not in general physically realizable for arbitrary
quantum operations and thus is not included here as an option.

## Input
### nQubits
Number of qubits to pass to each operation.
### actual
Operation to be tested.
### expected
Operation defining the expected behavior for the operation under test.
## Output
True if operations are equal, false otherwise.
