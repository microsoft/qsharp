# ApproximateFactorial function

`function ApproximateFactorial(n : Int) : Double`

## Summary
Returns an approximate factorial of a given number.

## Description
Returns the factorial as `Double`, given an input `n`.
The domain of inputs for this function is `n <= 169`.

## Remarks
For n > 10, this function uses the Ramanujan approximation with a
relative error of the order of 1 / n⁵.

## Input
### n
The number to take the approximate factorial of. Must not be negative.

## Output
The approximate factorial of `n`.

## See Also
- Microsoft.Quantum.Math.FactorialI
- Microsoft.Quantum.Math.FactorialL
