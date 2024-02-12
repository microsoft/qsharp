---
uid: Microsoft.Quantum.ResourceEstimation.BeginEstimateCaching
title: BeginEstimateCaching function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.ResourceEstimation
qsharp.name: BeginEstimateCaching
qsharp.summary: Informs the resource estimator of the start of the code fragment
for which estimates caching can be done. This function
is only available when using resource estimator execution target.
---

# BeginEstimateCaching function

Namespace: [Microsoft.Quantum.ResourceEstimation](xref:Microsoft.Quantum.ResourceEstimation)

```qsharp
function BeginEstimateCaching(name : String, variant : Int) : Bool
```

## Summary
Informs the resource estimator of the start of the code fragment
for which estimates caching can be done. This function
is only available when using resource estimator execution target.

## Input
### name
The name of the code fragment. Used to distinguish it from other code fragments.
Typically this is the name of the operation for which estimates can be cached.
### variant
Specific variant of the execution. Cached estimates can only be reused if the
variant for which they were collected and the current variant is the same.

## Output
`true` indicated that the cached estimates are not yet avialable and the code fragment
needs to be executed in order to collect and cache estimates.
`false` indicates if cached estimates have been incorporated into the overall costs
and the code fragment should be skipped.
