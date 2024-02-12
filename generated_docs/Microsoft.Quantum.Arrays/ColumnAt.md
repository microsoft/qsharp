---
uid Microsoft.Quantum.Arrays.ColumnAt
title: ColumnAt function
ms.date: 02/12/2024 12:00:00 AM
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Arrays
qsharp.name: ColumnAt
qsharp.summary: Extracts a column from a matrix.
---

# ColumnAt function

Namespace: [Microsoft.Quantum.Arrays](xref:Microsoft.Quantum.Arrays)

```qsharp
function ColumnAt<'T>(column : Int, matrix : 'T[][]) : 'T[]
```

## Summary
Extracts a column from a matrix.

## Description
This function extracts a column in a matrix in row-wise order.
Extracting a row corresponds to element access of the first index
and therefore requires no further treatment.

## Type Parameters
### 'T
The type of each element of `matrix`.

## Input
### column
Column of the matrix
### matrix
2-dimensional matrix in row-wise order

## Example
```qsharp
let matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
let column = ColumnAt(0, matrix);
// same as: column = [1, 4, 7]
```

## See Also
- Microsoft.Quantum.Arrays.Transposed
- Microsoft.Quantum.Arrays.Diagonal
