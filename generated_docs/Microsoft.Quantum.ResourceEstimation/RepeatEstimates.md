---
uid Microsoft.Quantum.ResourceEstimation.RepeatEstimates
title: RepeatEstimates operation
ms.date: todo
ms.topic: managed-reference
qsharp.kind: opeartion
qsharp.namespace: Microsoft.Quantum.ResourceEstimation
qsharp.name: RepeatEstimates
qsharp.summary: Instructs the resource estimator to assume that the resources from the
call of this operation until a call to `Adjoint RepeatEstimates` are
accounted for `count` times, without the need to execute the code that many
times.
---

# RepeatEstimates operation

Namespace: [Microsoft.Quantum.ResourceEstimation](xref:Microsoft.Quantum.ResourceEstimation)

Instructs the resource estimator to assume that the resources from the
call of this operation until a call to `Adjoint RepeatEstimates` are
accounted for `count` times, without the need to execute the code that many
times.
```qsharp
operation RepeatEstimates(count : Int) : Unit is Adj
```

## Summary

Instructs the resource estimator to assume that the resources from the
call of this operation until a call to `Adjoint RepeatEstimates` are
accounted for `count` times, without the need to execute the code that many
times.

## Input
### count
Assumed number of repetitions, factor to multiply the cost with
