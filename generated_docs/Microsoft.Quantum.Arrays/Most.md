# Most function

`function Most<'T>(array : 'T[]) : 'T[]`

## Summary
Creates an array that is equal to an input array except that the last array
element is dropped.

## Type Parameters
### 'T
The type of the array elements.

## Input
### array
An array whose first to second-to-last elements are to form the output array.

## Output
An array containing the elements `array[0..Length(array) - 2]`.
