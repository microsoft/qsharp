---
uid: Microsoft.Quantum.Convert.BigIntAsBoolArray
title: BigIntAsBoolArray function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Convert
qsharp.name: BigIntAsBoolArray
qsharp.summary: Produces a binary representation of a non-negative BigInt, using the
little-endian representation for the returned array.
---

# BigIntAsBoolArray function

Namespace: [Microsoft.Quantum.Convert](xref:Microsoft.Quantum.Convert)

```qsharp
function BigIntAsBoolArray(number : BigInt, bits : Int) : Bool[]
```

## Summary
Produces a binary representation of a non-negative BigInt, using the
little-endian representation for the returned array.

## Input
### number
A non-negative BigInt to be converted to an array of Boolean values.
### bits
The number of bits in the binary representation of `number`.

## Output
An array of Boolean values representing `number`.

## Remarks
The input `bits` must be non-negative.
The input `number` must be between 0 and 2^bits - 1.
