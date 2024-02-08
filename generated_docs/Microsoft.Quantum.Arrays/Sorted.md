# function Sorted<'T>(comparison : (('T, 'T) -> Bool), array : 'T[]) : 'T[]

## Summary
Given an array, returns the elements of that array sorted by a given
comparison function.

## Type Parameters
### 'T
The type of each element of `array`.

## Input
### comparison
A function that compares two elements such that `a` is considered to
be less than or equal to `b` if `comparison(a, b)` is `true`.
### array
The array to be sorted.

## Output
An array containing the same elements as `array`, such that for all
elements `a` occurring earlier than elements `b`, `comparison(a, b)`
is `true`.

## Example
The following snippet sorts an array of integers to occur in ascending
order:
```qsharp
let sortedArray = Sorted(LessThanOrEqualI, [3, 17, 11, -201, -11]);
```

## Remarks
The function `comparison` is assumed to be transitive, such that
if `comparison(a, b)` and `comparison(b, c)`, then `comparison(a, c)`
is assumed. If this property does not hold, then the output of this
function may be incorrect.
