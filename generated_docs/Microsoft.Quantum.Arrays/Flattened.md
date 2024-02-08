# Flattened function

`function Flattened<'T>(arrays : 'T[][]) : 'T[]`

## Summary
Given an array of arrays, returns the concatenation of all arrays.

## Type Parameters
### 'T
The type of `array` elements.

## Input
### arrays
Array of arrays.

## Output
Concatenation of all arrays.

## Example
```qsharp
let flattened = Flattened([[1, 2], [3], [4, 5, 6]]);
// flattened = [1, 2, 3, 4, 5, 6]
```
