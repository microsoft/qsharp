# function LogGammaD(x : Double) : Double

## Summary
Returns the natural logarithm of the gamma function (aka the log-gamma
function).

## Description
The gamma function Γ(x) generalizes the factorial function
to the positive real numbers and is defined as
integral from 0 to ∞ of t¹⁻ˣ⋅e⁻ᵗ𝑑t

The gamma function has the property that for all positive real numbers
x, Γ(x + 1) = x⋅Γ(x), such that the factorial function
is a special case of Γ, n! = Γ(n + 1) for all natural numbers n.

## Input
### x
The point x at which the log-gamma function is to be evaluated.

## Output
The value ㏑(Γ(x)).
