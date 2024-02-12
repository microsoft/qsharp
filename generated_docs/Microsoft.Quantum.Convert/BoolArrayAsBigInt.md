---
uid Microsoft.Quantum.Convert.BoolArrayAsBigInt
title: BoolArrayAsBigInt function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Convert
qsharp.name: BoolArrayAsBigInt
qsharp.summary: Converts an array of Boolean values into a non-negative BigInt, interpreting the
array as a binary representation in little-endian format.
---

# BoolArrayAsBigInt function

Namespace: [Microsoft.Quantum.Convert](xref:Microsoft.Quantum.Convert)

```qsharp
function BoolArrayAsBigInt(boolArray : Bool[]) : BigInt
```

## Summary
Converts an array of Boolean values into a non-negative BigInt, interpreting the
array as a binary representation in little-endian format.

## Input
### boolArray
An array of Boolean values representing the binary digits of a BigInt.

## Output
A BigInt represented by `boolArray`.

## Remarks
The function interprets the array in little-endian format, where the first
element of the array represents the least significant bit.
The input `boolArray` should not be empty.
