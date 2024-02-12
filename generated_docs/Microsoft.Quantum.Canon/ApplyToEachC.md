---
uid Microsoft.Quantum.Canon.ApplyToEachC
title: ApplyToEachC operation
ms.date: todo
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.Canon
qsharp.name: ApplyToEachC
qsharp.summary: Applies an operation to each element in a register.
The modifier `C` indicates that the single-element operation is controllable.
---

# ApplyToEachC operation

Namespace: [Microsoft.Quantum.Canon](xref:Microsoft.Quantum.Canon)

Applies an operation to each element in a register.
The modifier `C` indicates that the single-element operation is controllable.
```qsharp
operation ApplyToEachC<'T>(singleElementOperation : ('T => Unit is Param<1>), register : 'T[]) : Unit is Ctl
```

## Summary
Applies an operation to each element in a register.
The modifier `C` indicates that the single-element operation is controllable.

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
