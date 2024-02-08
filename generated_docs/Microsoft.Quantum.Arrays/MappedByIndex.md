# function MappedByIndex<'T, 'U>(mapper : ((Int, 'T) -> 'U), array : 'T[]) : 'U[]

## Summary
Given an array and a function that is defined
for the indexed elements of the array, returns a new array that consists
of the images of the original array under the function.

## Type Parameters
### 'T
The type of `array` elements.
### 'U
The result type of the `mapper` function.

## Input
### mapper
A function from `(Int, 'T)` to `'U` that is used to map elements
and their indices.
### array
An array of elements over `'T`.

## Output
An array `'U[]` of elements that are mapped by the `mapper` function.

## Example
The following two lines are equivalent:
```qsharp
let array = MappedByIndex(f, [x0, x1, x2]);
```
and
```qsharp
let array = [f(0, x0), f(1, x1), f(2, x2)];
```

## See Also
- Microsoft.Quantum.Arrays.Mapped
