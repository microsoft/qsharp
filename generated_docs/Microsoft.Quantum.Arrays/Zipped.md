# function Zipped<'T, 'U>(left : 'T[], right : 'U[]) : ('T, 'U)[]

## Summary
Given two arrays, returns a new array of pairs such that each pair
contains an element from each original array.

## Type Parameters
### 'T
The type of the left array elements.
### 'U
The type of the right array elements.

## Input
### left
An array containing values for the first element of each tuple.
### right
An array containing values for the second element of each tuple.

## Output
An array containing pairs of the form `(left[index], right[index])` for
each `index`. If the two arrays are not of equal length, the output will
be as long as the shorter of the inputs.

## Example
```qsharp
let left = [1, 3, 71];
let right = [false, true];
let pairs = Zipped(left, right); // [(1, false), (3, true)]
```

## See Also
- Microsoft.Quantum.Arrays.Unzipped
