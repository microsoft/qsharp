---
uid Microsoft.Quantum.Canon.ApplyToEachCA
title: ApplyToEachCA operation
ms.date: 02/12/2024
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Canon
qsharp.name: ApplyToEachCA
qsharp.summary: Applies an operation to each element in a register.
The modifier `CA` indicates that the single-element operation is controllable and adjointable.
---

# ApplyToEachCA operation

Namespace: [Microsoft.Quantum.Canon](xref:Microsoft.Quantum.Canon)

```qsharp
operation ApplyToEachCA<'T>(singleElementOperation : ('T => Unit is Param<1>), register : 'T[]) : Unit is Adj + Ctl
```

## Summary
Applies an operation to each element in a register.
The modifier `CA` indicates that the single-element operation is controllable and adjointable.

## Input
### singleElementOperation
Operation to apply to each element.
### register
Array of elements on which to apply the given operation.

## Type Parameters
### 'T
The target on which the operation acts.

## Example
Prepare a three-qubit |+⟩ state:
```qsharp
use register = Qubit[3];
ApplyToEach(H, register);
```

## See Also
- Microsoft.Quantum.Canon.ApplyToEach
