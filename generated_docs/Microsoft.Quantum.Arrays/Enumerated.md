# function Enumerated<'TElement>(array : 'TElement[]) : (Int, 'TElement)[]

## Summary
Given an array, returns a new array containing elements of the original
array along with the indices of each element.

## Type Parameters
### 'TElement
The type of elements of the array.

## Input
### array
An array whose elements are to be enumerated.

## Output
A new array containing elements of the original array along with their
indices.

## Example
The following `for` loops are equivalent:
```qsharp
for (idx in IndexRange(array)) {
    let element = array[idx];
    ...
}
for ((idx, element) in Enumerated(array)) { ... }
```
