# function Subarray<'T>(locations : Int[], array : 'T[]) : 'T[]

## Summary
Takes an array and a list of locations and
produces a new array formed from the elements of the original
array that match the given locations.

## Remarks
If `locations` contains repeated elements, the corresponding elements
of `array` will likewise be repeated.

## Type Parameters
### 'T
The type of `array` elements.

## Input
### locations
A list of locations in the input array that is used to define the subarray.
### array
An array from which a subarray will be generated.

## Output
An array `out` of elements whose locations correspond to the subarray,
such that `out[index] == array[locations[index]]`.

## Example

```qsharp
let array = [1, 2, 3, 4];
let permutation = Subarray([3, 0, 2, 1], array); // [4, 1, 3, 2]
let duplicates = Subarray([1, 2, 2], array);     // [2, 3, 3]
```
