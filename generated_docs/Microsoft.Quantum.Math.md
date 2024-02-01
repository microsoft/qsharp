# Summary
Represents the ratio of the circumference of a circle to its diameter.

# Ouptut
A double-precision approximation of the the circumference of a circle
to its diameter, π ≈ 3.14159265358979323846.

# See Also
- Microsoft.Quantum.Math.E
---
function PI() : Double

---

# Summary
Returns the natural logarithmic base to double-precision.

# Output
A double-precision approximation of the natural logarithic base,
e ≈ 2.7182818284590452354.

# See Also
- Microsoft.Quantum.Math.PI
---
function E() : Double

---

# Summary
Returns the natural logarithm of 2.

# Output
Returns a `Double` equal to 0.6931471805599453.
---
function LogOf2() : Double

---

# Summary
Returns whether a given floating-point value is not a number (i.e. is
NaN).

# Input
## d
A floating-point value to be checked.

# Output
`true` if and only if `d` is not a number.
---
function IsNaN(d : Double) : Bool

---

# Summary
Returns whether a given floating-point value is either positive or
negative infinity.

# Input
## d
The floating-point value to be checked.

# Ouput
`true` if and only if `d` is either positive or negative infinity.

# Remarks
`NaN` is not a number, and is thus neither a finite number nor
is it infinite. As such, `IsInfinite(0.0 / 0.0)` returns `false`.
To check if a value is `NaN`, use `IsNaN(d)`.

Note that even though this function returns `true` for both
positive and negative infinities, these values can still be
discriminated by checking `d > 0.0` and `d < 0.0`.

# Example
```qsharp
Message($"{IsInfinite(42.0)}"); // false
Message($"{IsInfinite(0.0 / 0.0)}"); // false
Message($"{IsInfinite(-1.0 / 0.0}"); // true
```

# See Also
- Microsoft.Quantum.Math.IsNaN
---
function IsInfinite(d : Double) : Bool

---

# Summary
Returns -1, 0 or +1 that indicates the sign of a number.
---
function SignI(a : Int) : Int

---

# Summary
Returns -1, 0 or +1 that indicates the sign of a number.
---
function SignD(a : Double) : Int

---

# Summary
Returns -1, 0 or +1 that indicates the sign of a number.
---
function SignL(a : BigInt) : Int

---

# Summary
Returns the absolute value of an integer.
---
function AbsI(a : Int) : Int

---

# Summary
Returns the absolute value of a double-precision floating-point number.
---
function AbsD(a : Double) : Double

---

# Summary
---
function AbsL(a : BigInt) : BigInt

---

# Summary
Returns the larger of two specified numbers.
---
function MaxI(a : Int, b : Int) : Int

---

# Summary
Returns the larger of two specified numbers.
---
function MaxD(a : Double, b : Double) : Double

---

# Summary
Returns the larger of two specified numbers.
---
function MaxL(a : BigInt, b : BigInt) : BigInt

---

# Summary
Returns the smaller of two specified numbers.
---
function MinI(a : Int, b : Int) : Int

---

# Summary
Returns the smaller of two specified numbers.
---
function MinD(a : Double, b : Double) : Double

---

# Summary
Returns the smaller of two specified numbers.
---
function MinL(a : BigInt, b : BigInt) : BigInt

---

# Summary
Given an array of integers, returns the largest element.

# Input
## values
An array to take the maximum of.

# Output
The largest element of `values`.
---
function Max(values : Int[]) : Int

---

# Summary
Given an array of integers, returns the smallest element.

# Input
## values
An array to take the minimum of.

# Output
The smallest element of `values`.
---
function Min(values : Int[]) : Int

---

# Summary
Returns the angle whose cosine is the specified number.
---
function ArcCos(x : Double) : Double

---

# Summary
Returns the angle whose sine is the specified number.
---
function ArcSin(y : Double) : Double

---

# Summary
Returns the angle whose tangent is the specified number.
---
function ArcTan(d : Double) : Double

---

# Summary
Returns the angle whose tangent is the quotient of two specified numbers.
---
function ArcTan2(y : Double, x : Double) : Double

---

# Summary
Returns the cosine of the specified angle.
---
function Cos(theta : Double) : Double

---

# Summary
Returns the hyperbolic cosine of the specified angle.
---
function Cosh(d : Double) : Double

---

# Summary
Returns the sine of the specified angle.
---
function Sin(theta : Double) : Double

---

# Summary
Returns the hyperbolic sine of the specified angle.
---
function Sinh(d : Double) : Double

---

# Summary
Returns the tangent of the specified angle.
---
function Tan(d : Double) : Double

---

# Summary
Returns the hyperbolic tangent of the specified angle.
---
function Tanh(d : Double) : Double

---

# Summary
Computes the inverse hyperbolic cosine of a number.
---
function ArcCosh(x : Double) : Double

---

# Summary
Computes the inverse hyperbolic sine of a number.
---
function ArcSinh(x : Double) : Double

---

# Summary
Computes the inverse hyperbolic tangent of a number.
---
function ArcTanh(x : Double) : Double

---

# Summary
Returns the square root of a specified number.
---
function Sqrt(d : Double) : Double

---

# Summary
Returns the natural (base _e_) logarithm of a specified number.
---
function Log(input : Double) : Double

---

# Summary
Returns the base-10 logarithm of a specified number.
---
function Log10(input : Double) : Double

---

# Summary
Computes the base-2 logarithm of a number.
---
function Lg(input : Double) : Double

---

# Summary
Returns the integral part of a number.
For example: Truncate(3.7) = 3; Truncate(-3.7) = -3
---
function Truncate(value : Double) : Int

---

# Summary
Returns the smallest integer greater than or equal to the specified number.
For example: Ceiling(3.1) = 4; Ceiling(-3.7) = -3
---
function Ceiling(value : Double) : Int

---

# Summary
Returns the largest integer less than or equal to the specified number.
For example: Floor(3.7) = 3; Floor(-3.1) = -4
---
function Floor(value : Double) : Int

---

# Summary
Returns the nearest integer to the specified number.
For example: Floor(3.7) = 4; Floor(-3.7) = -4
---
function Round(value : Double) : Int

---

# Summary
Divides one Integer value by another, returns the result and the remainder as a tuple.
---
function DivRemI(dividend : Int, divisor : Int) : (Int, Int)

---

# Summary
Divides one BigInteger value by another, returns the result and the remainder as a tuple.
---
function DivRemL(dividend : BigInt, divisor : BigInt) : (BigInt, BigInt)

---

# Summary
Computes the canonical residue of `value` modulo `modulus`.
The result is always in the range 0..modulus-1 even for negative numbers.
---
function ModulusI(value : Int, modulus : Int) : Int

---

# Summary
Computes the canonical residue of `value` modulo `modulus`.
The result is always in the range 0..modulus-1 even for negative numbers.
---
function ModulusL(value : BigInt, modulus : BigInt) : BigInt

---

# Summary
Returns an integer raised to a given power, with respect to a given
modulus. I.e. (expBase^power) % modulus.
---
function ExpModI(expBase : Int, power : Int, modulus : Int) : Int

---

# Summary
Returns an integer raised to a given power, with respect to a given
modulus. I.e. (expBase^power) % modulus.
---
function ExpModL(expBase : BigInt, power : BigInt, modulus : BigInt) : BigInt

---

# Summary
Returns the multiplicative inverse of a modular integer.

# Description
This will calculate the multiplicative inverse of a
modular integer `b` such that `a • b = 1 (mod modulus)`.
---
function InverseModI(a : Int, modulus : Int) : Int

---

# Summary
Returns the multiplicative inverse of a modular integer.

# Description
This will calculate the multiplicative inverse of a
modular integer `b` such that `a • b = 1 (mod modulus)`.
---
function InverseModL(a : BigInt, modulus : BigInt) : BigInt

---

# Summary
Computes the greatest common divisor of two integers.
Note: GCD is always positive except that GCD(0,0)=0.
---
function GreatestCommonDivisorI(a : Int, b : Int) : Int

---

# Summary
Computes the greatest common divisor of two integers.
Note: GCD is always positive except that GCD(0,0)=0.
---
function GreatestCommonDivisorL(a : BigInt, b : BigInt) : BigInt

---

# Summary
Returns a tuple (u,v) such that u*a+v*b=GCD(a,b)
Note: GCD is always positive except that GCD(0,0)=0.
---
function ExtendedGreatestCommonDivisorI(a : Int, b : Int) : (Int, Int)

---

# Summary
Returns a tuple (u,v) such that u*a+v*b=GCD(a,b)
Note: GCD is always positive except that GCD(0,0)=0.
---
function ExtendedGreatestCommonDivisorL(a : BigInt, b : BigInt) : (BigInt, BigInt)

---

# Summary
Returns if two integers are co-prime.

# Description
Returns true if a and b are co-prime and false otherwise.

# Input
## a
the first number of which co-primality is being tested
## b
the second number of which co-primality is being tested

# Output
True, if a and b are co-prime (e.g. their greatest common divisor is 1),
and false otherwise
---
function IsCoprimeI(a : Int, b : Int) : Bool

---

# Summary
Returns if two integers are co-prime.

# Description
Returns true if a and b are co-prime and false otherwise.

# Input
## a
the first number of which co-primality is being tested
## b
the second number of which co-primality is being tested

# Output
True, if a and b are co-prime (e.g. their greatest common divisor is 1),
and false otherwise
---
function IsCoprimeL(a : BigInt, b : BigInt) : Bool

---

# Summary
Finds the continued fraction convergent closest to `fraction`
with the denominator less or equal to `denominatorBound`
Using process similar to this: https://nrich.maths.org/1397
---
function ContinuedFractionConvergentI(fraction : (Int, Int), denominatorBound : Int) : (Int, Int)

---

# Summary
Finds the continued fraction convergent closest to `fraction`
with the denominator less or equal to `denominatorBound`
Using process similar to this: https://nrich.maths.org/1397
---
function ContinuedFractionConvergentL(fraction : (BigInt, BigInt), denominatorBound : BigInt) : (BigInt, BigInt)

---

# Summary
Computes the modulus between two real numbers.

# Input
## value
A real number x to take the modulus of.
## modulo
A real number to take the modulus of x with respect to.
## minValue
The smallest value to be returned by this function.

# Example
```qsharp
    // Returns 3 π / 2.
    let y = RealMod(5.5 * PI(), 2.0 * PI(), 0.0);
    // Returns -1.2, since +3.6 and -1.2 are 4.8 apart on the real line,
    // which is a multiple of 2.4.
    let z = RealMod(3.6, 2.4, -1.2);
```
---
function RealMod(value : Double, modulo : Double, minValue : Double) : Double

---

# Summary
For a non-negative integer `a`, returns the number of bits required to represent `a`.
NOTE: This function returns the smallest n such that a < 2^n.
---
function BitSizeI(a : Int) : Int

---

# Summary
For a non-negative integer `a`, returns the number of bits required to represent `a`.
NOTE: This function returns the smallest n such that a < 2^n.
---
function BitSizeL(a : BigInt) : Int

---

# Summary
For a non-zero integer `a`, returns the number of trailing zero bits
in the binary representation of `a`.
---
function TrailingZeroCountI(a : Int) : Int

---

# Summary
For a non-zero integer `a`, returns the number of trailing zero bits
in the binary representation of `a`.
---
function TrailingZeroCountL(a : BigInt) : Int

---

# Summary
Returns the number of 1 bits in the binary representation of integer `n`.
---
function HammingWeightI(n : Int) : Int

---

# Summary
Returns the factorial of a given number.

# Description
Returns the factorial of a given nonnegative integer n, where 0 ≤ n ≤ 20.

# Input
## n
The number to take the factorial of.

# Output
The factorial of `n`.

# Remarks
For inputs greater than 20, please use `Microsoft.Quantum.Math.FactorialL`.

# See Also
- Microsoft.Quantum.Math.FactorialL
- Microsoft.Quantum.Math.ApproximateFactorial
---
function FactorialI(n : Int) : Int

---

# Summary
Returns the factorial of a given number.

# Input
## n
The number to take the factorial of.

# Output
The factorial of `n`.

# See Also
- Microsoft.Quantum.Math.FactorialI
- Microsoft.Quantum.Math.ApproximateFactorial
---
function FactorialL(n : Int) : BigInt

---

# Summary
Returns an approximate factorial of a given number.

# Description
Returns the factorial as `Double`, given an input `n`.
The domain of inputs for this function is `n <= 169`.

# Remarks
For n > 10, this function uses the Ramanujan approximation with a
relative error of the order of 1 / n⁵.

# Input
## n
The number to take the approximate factorial of. Must not be negative.

# Output
The approximate factorial of `n`.

# See Also
- Microsoft.Quantum.Math.FactorialI
- Microsoft.Quantum.Math.FactorialL
---
function ApproximateFactorial(n : Int) : Double

---

# Summary
Returns the natural logarithm of the gamma function (aka the log-gamma
function).

# Description
The gamma function Γ(x) generalizes the factorial function
to the positive real numbers and is defined as
integral from 0 to ∞ of t¹⁻ˣ⋅e⁻ᵗ𝑑t

The gamma function has the property that for all positive real numbers
x, Γ(x + 1) = x⋅Γ(x), such that the factorial function
is a special case of Γ, n! = Γ(n + 1) for all natural numbers n.

# Input
## x
The point x at which the log-gamma function is to be evaluated.

# Output
The value ㏑(Γ(x)).
---
function LogGammaD(x : Double) : Double

---

# Summary
Returns the approximate natural logarithm of the factorial of a given
integer.

# Input
## n
The number to take the log-factorial of.

# Output
The natural logarithm of the factorial of the provided input.

# See Also
- Microsoft.Quantum.Math.ApproximateFactorial
- Microsoft.Quantum.Math.FactorialI
- Microsoft.Quantum.Math.FactorialL
---
function LogFactorialD(n : Int) : Double

---

# Summary
Returns the approximate binomial coefficient of two integers.

# Description
Given two integers n and k, returns the binomial coefficient
binom(n, k), also known as n-choose-k. Computed approximately.

# Input
## n
The first of the two integers to compute the binomial coefficient of.
## k
The second of the two integers to compute the binomial coefficient of.

# Output
The binomial coefficient n-choose-k.
---
function Binom(n : Int, k : Int) : Int

---

# Summary
Returns the squared 2-norm of a vector.

# Description
Returns the squared 2-norm of a vector; that is, given an input
x̄, returns ∑xᵢ.

# Input
## array
The vector whose squared 2-norm is to be returned.

# Output
The squared 2-norm of `array`.
---
function SquaredNorm(array : Double[]) : Double

---

# Summary
Returns the `L(p)` norm of a vector of `Double`s.

That is, given an array x of type `Double[]`, this returns the p-norm
|x̄|ₚ= (∑(xᵢ)ᵖ)¹ᐟᵖ.

# Input
## p
The exponent p in the p-norm.

# Output
The p-norm |x̄|ₚ.
---
function PNorm(p : Double, array : Double[]) : Double

---

# Summary
Normalizes a vector of `Double`s in the `L(p)` norm.

That is, given an array x of type `Double[]`, this returns an array where
all elements are divided by the p-norm |x̄|ₚ.
Function leaves array with norm 0 unchanged.

# Input
## p
The exponent p in the p-norm.

# Output
The array x normalized by the p-norm |x̄|ₚ.

# See Also
- PNorm
---
function PNormalized(p : Double, array : Double[]) : Double[]

---

# Summary
Represents a complex number by its real and imaginary components.
The first element of the tuple is the real component,
the second one - the imaginary component.

# Example
The following snippet defines the imaginary unit 𝑖 = 0 + 1𝑖:
```qsharp
let imagUnit = Complex(0.0, 1.0);
```
---
newtype Complex = (Real: Double, Imag: Double)

---

# Summary
Represents a complex number in polar form.
The polar representation of a complex number is c = r⋅𝑒^(t𝑖).

# Named Items
## Magnitude
The absolute value r>0 of c.
## Argument
The phase t ∈ ℝ of c.
---
newtype ComplexPolar = (Magnitude: Double, Argument: Double)

---

# Summary
Returns the squared absolute value of a complex number of type
`Complex`.

# Input
## input
Complex number c = x + y𝑖.

# Output
Squared absolute value |c|² = x² + y².
---
function AbsSquaredComplex(input : Complex) : Double

---

# Summary
Returns the absolute value of a complex number of type
`Complex`.

# Input
## input
Complex number c = x + y𝑖.

# Output
Absolute value |c| = √(x² + y²).
---
function AbsComplex(input : Complex) : Double

---

# Summary
Returns the phase of a complex number of type
`Complex`.

# Input
## input
Complex number c = x + y𝑖.

# Output
Phase Arg(c) = ArcTan(y,x) ∈ (-𝜋,𝜋].
---
function ArgComplex(input : Complex) : Double

---

# Summary
Returns the squared absolute value of a complex number of type
`ComplexPolar`.

# Input
## input
Complex number c = r⋅𝑒^(t𝑖).

# Output
Squared absolute value |c|² = r².
---
function AbsSquaredComplexPolar(input : ComplexPolar) : Double

---

# Summary
Returns the absolute value of a complex number of type
`ComplexPolar`.

# Input
## input
Complex number c = r⋅𝑒^(t𝑖).

# Output
Absolute value |c| = r.
---
function AbsComplexPolar(input : ComplexPolar) : Double

---

# Summary
Returns the phase of a complex number of type `ComplexPolar`.

# Input
## input
Complex number c = r⋅𝑒^(t𝑖).

# Output
Phase Arg(c) = t.
---
function ArgComplexPolar(input : ComplexPolar) : Double

---

# Summary
Returns the unary negation of an input of type `Complex`.

# Input
## input
A value whose negation is to be returned.

# Output
The unary negation of `input`.
---
function NegationC(input : Complex) : Complex

---

# Summary
Returns the unary negation of an input of type `ComplexPolar`

# Input
## input
A value whose negation is to be returned.

# Output
The unary negation of `input`.
---
function NegationCP(input : ComplexPolar) : ComplexPolar

---

# Summary
Returns the sum of two inputs of type `Complex`.

# Input
## a
The first input a to be summed.
## b
The second input b to be summed.

# Output
The sum a + b.
---
function PlusC(a : Complex, b : Complex) : Complex

---

# Summary
Returns the sum of two inputs of type `ComplexPolar`.

# Input
## a
The first input a to be summed.
## b
The second input b to be summed.

# Output
The sum a + b.
---
function PlusCP(a : ComplexPolar, b : ComplexPolar) : ComplexPolar

---

# Summary
Returns the difference between two inputs of type `Complex`.

# Input
## a
The first input a to be subtracted.
## b
The second input b to be subtracted.

# Output
The difference a - b.
---
function MinusC(a : Complex, b : Complex) : Complex

---

# Summary
Returns the difference between two inputs of type `ComplexPolar`.

# Input
## a
The first input a to be subtracted.
## b
The second input b to be subtracted.

# Output
The difference a - b.
---
function MinusCP(a : ComplexPolar, b : ComplexPolar) : ComplexPolar

---

# Summary
Returns the product of two inputs of type `Complex`.

# Input
## a
The first input a to be multiplied.
## b
The second input b to be multiplied.

# Output
The product a⋅b.
---
function TimesC(a : Complex, b : Complex) : Complex

---

# Summary
Returns the product of two inputs of type `ComplexPolar`.

# Input
## a
The first input a to be multiplied.
## b
The second input b to be multiplied.

# Output
The product a⋅b.
---
function TimesCP(a : ComplexPolar, b : ComplexPolar) : ComplexPolar

---

# Summary
Returns a number raised to a given power of type `Complex`.
Note that this is a multi-valued function, but only one value is returned.

# Input
## a
The number a that is to be raised.
## power
The power b to which a should be raised.

# Output
The power a^b
---
function PowC(a : Complex, power : Complex) : Complex

---

# Summary
Returns a number raised to a given power of type `ComplexPolar`.
Note that this is a multi-valued function, but only one value is returned.

# Input
## a
The number a that is to be raised.
## power
The power b to which a should be raised.

# Output
The power a^b
---
function PowCP(a : ComplexPolar, power : ComplexPolar) : ComplexPolar

---

# Summary
Returns the quotient of two inputs of type `Complex`.

# Input
## a
The first input a to be divided.
## b
The second input b to be divided.

# Output
The quotient a / b.
---
function DividedByC(a : Complex, b : Complex) : Complex

---

# Summary
Returns the quotient of two inputs of type `ComplexPolar`.

# Input
## a
The first input a to be divided.
## b
The second input b to be divided.

# Output
The quotient a / b.
---
function DividedByCP(a : ComplexPolar, b : ComplexPolar) : ComplexPolar

---

# Summary
Returns the smallest representable number for specific fixed point dimensions.

# Input
## integerBits
Number of integer bits (including the sign bit).
## fractionalBits
Number of fractional bits.

# Remark
The value can be computed as -2^(p-1), where p is the number of integer bits.
---
function SmallestFixedPoint(integerBits : Int, fractionalBits : Int) : Double

---

# Summary
Returns the largest representable number for specific fixed point dimensions.

# Input
## integerBits
Number of integer bits (including the sign bit).
## fractionalBits
Number of fractional bits.

# Remark
The value can be computed as 2^(p-1) - 2^(-q), where p
is the number of integer bits and q is the number of fractional bits.
---
function LargestFixedPoint(integerBits : Int, fractionalBits : Int) : Double
