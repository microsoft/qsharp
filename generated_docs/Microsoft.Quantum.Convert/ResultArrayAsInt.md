# function ResultArrayAsInt(results : Result[]) : Int

## Summary
Produces a non-negative integer from a string of Results in little-endian format.

## Input
### results
Results in binary representation of number.

## Output
A non-negative integer

## Example
```qsharp
// The following returns 1
let int1 = ResultArrayAsInt([One,Zero])
```
