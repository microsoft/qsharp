// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Math {
    open Microsoft.Quantum.Diagnostics;

    //
    // Sign, Abs, Min, Max, etc.
    //

    /// # Summary
    /// Returns -1, 0 or +1 that indicates the sign of a number.
    function SignI (a : Int) : Int {
        if   (a < 0) { -1 }
        elif (a > 0) { +1 }
        else         { 0 }
    }

    /// # Summary
    /// Returns -1, 0 or +1 that indicates the sign of a number.
    function SignD (a : Double) : Int {
        if   (a < 0.0) { -1 }
        elif (a > 0.0) { +1 }
        else           {  0 }
    }

    /// # Summary
    /// Returns -1, 0 or +1 that indicates the sign of a number.
    function SignL (a : BigInt) : Int {
        if   (a < 0L) { -1 }
        elif (a > 0L) { +1 }
        else          {  0 }
    }

    /// # Summary
    /// Returns the absolute value of an integer.
    function AbsI (a : Int) : Int {
        a < 0 ? -a | a
    }

    /// # Summary
    /// Returns the absolute value of a double-precision floating-point number.
    function AbsD (a : Double) : Double {
        a < 0.0 ? -a | a
    }

    /// # Summary
    function AbsL (a : BigInt) : BigInt {
        a < 0L ? -a | a
    }

    /// # Summary
    /// Returns the larger of two specified numbers.
    function MaxI(a : Int, b : Int) : Int {
        a > b ? a | b
    }

    /// # Summary
    /// Returns the larger of two specified numbers.
    function MaxD(a : Double, b : Double) : Double {
        a > b ? a | b
    }

    /// # Summary
    /// Returns the larger of two specified numbers.
    function MaxL (a : BigInt, b : BigInt) : BigInt {
        a > b ? a | b
    }

    /// # Summary
    /// Returns the smaller of two specified numbers.
    function MinI (a : Int, b : Int) : Int {
        a < b ? a | b
    }

    /// # Summary
    /// Returns the smaller of two specified numbers.
    function MinD (a : Double, b : Double) : Double {
        a < b ? a | b
    }

    /// # Summary
    /// Returns the smaller of two specified numbers.
    function MinL(a : BigInt, b : BigInt) : BigInt {
        a < b ? a | b
    }

    //
    // Trigonometric functions
    //

    /// # Summary
    /// Represents the ratio of the circumference of a circle to its diameter.
    ///
    /// # Ouptut
    /// A double-precision approximation of the the circumference of a circle
    /// to its diameter, π ≈ 3.14159265358979323846.
    ///
    /// # See Also
    /// - Microsoft.Quantum.Math.E
    function PI() : Double {
        3.14159265358979323846
    }

    /// # Summary
    /// Returns the natural logarithmic base to double-precision.
    ///
    /// # Output
    /// A double-precision approximation of the natural logarithic base,
    /// e ≈ 2.7182818284590452354.
    ///
    /// # See Also
    /// - Microsoft.Quantum.Math.PI
    function E() : Double {
        2.7182818284590452354
    }

    /// # Summary
    /// Returns the angle whose cosine is the specified number.
    function ArcCos (x : Double) : Double {
        body intrinsic;
    }

    /// # Summary
    /// Returns the angle whose sine is the specified number.
    function ArcSin (y : Double) : Double {
        body intrinsic;
    }

    /// # Summary
    /// Returns the angle whose tangent is the specified number.
    function ArcTan (d : Double) : Double {
        body intrinsic;
    }

    /// # Summary
    /// Returns the angle whose tangent is the quotient of two specified numbers.
    function ArcTan2 (y : Double, x : Double) : Double {
        body intrinsic;
    }

    /// # Summary
    /// Returns the cosine of the specified angle.
    function Cos (theta : Double) : Double {
        body intrinsic;
    }

    /// # Summary
    /// Returns the hyperbolic cosine of the specified angle.
    function Cosh (d : Double) : Double {
        body intrinsic;
    }

    /// # Summary
    /// Returns the sine of the specified angle.
    function Sin (theta : Double) : Double {
        body intrinsic;
    }

    /// # Summary
    /// Returns the hyperbolic sine of the specified angle.
    function Sinh (d : Double) : Double {
        body intrinsic;
    }

    /// # Summary
    /// Returns the tangent of the specified angle.
    function Tan (d : Double) : Double {
        body intrinsic;
    }

    /// # Summary
    /// Returns the hyperbolic tangent of the specified angle.
    function Tanh (d : Double) : Double {
        body intrinsic;
    }

    /// # Summary
    /// Computes the inverse hyperbolic cosine of a number.
    function ArcCosh (x : Double) : Double {
        Log(x + Sqrt(x * x - 1.0))
    }

    /// # Summary
    /// Computes the inverse hyperbolic sine of a number.
    function ArcSinh (x : Double) : Double {
        Log(x + Sqrt(x * x + 1.0))
    }


    /// # Summary
    /// Computes the inverse hyperbolic tangent of a number.
    function ArcTanh (x : Double) : Double {
        Log((1.0 + x) / (1.0 - x)) * 0.5
    }

    //
    // Sqrt, Log, exp, etc.
    //

    /// # Summary
    /// Returns the square root of a specified number.
    function Sqrt(d : Double) : Double {
        body intrinsic;
    }

    /// # Summary
    /// Returns the natural (base _e_) logarithm of a specified number.
    function Log(input : Double) : Double {
        body intrinsic;
    }

    /// # Summary
    /// Returns the base-10 logarithm of a specified number.
    function Log10(input : Double) : Double {
        Log(input) / Log(10.0)
    }

    /// # Summary
    /// Computes the base-2 logarithm of a number.
    function Lg(input : Double) : Double {
        Log(input) / Log(2.0)
    }

    //
    // Truncation and Rounding
    //

    /// # Summary
    /// Returns the integral part of a number.
    /// For example: Truncate(3.7) = 3; Truncate(-3.7) = -3
    function Truncate(value : Double) : Int {
        body intrinsic;
    }

    internal function ExtendedTruncation(value : Double) : (Int, Double, Bool) {
        let truncated = Truncate(value);
        return (truncated, Microsoft.Quantum.Convert.IntAsDouble(truncated) - value, value >= 0.0);
    }

    /// # Summary
    /// Returns the smallest integer greater than or equal to the specified number.
    /// For example: Ceiling(3.1) = 4; Ceiling(-3.7) = -3
    function Ceiling(value : Double) : Int {
        let (truncated, remainder, isPositive) = ExtendedTruncation(value);
        if AbsD(remainder) <= 1e-15 {
            return truncated;
        } else {
            return isPositive ? truncated + 1 | truncated;
        }
    }

    /// # Summary
    /// Returns the largest integer less than or equal to the specified number.
    /// For example: Floor(3.7) = 3; Floor(-3.1) = -4
    function Floor(value : Double) : Int {
        let (truncated, remainder, isPositive) = ExtendedTruncation(value);
        if AbsD(remainder) <= 1e-15 {
            return truncated;
        } else {
            return isPositive ? truncated | truncated - 1;
        }
    }

    /// # Summary
    /// Returns the nearest integer to the specified number.
    /// For example: Floor(3.7) = 4; Floor(-3.7) = -4
    function Round(value : Double) : Int {
        let (truncated, remainder, isPositive) = ExtendedTruncation(value);
        if AbsD(remainder) <= 1e-15 {
            return truncated;
        } else {
            let abs = AbsD(remainder);
            return truncated + (abs <= 0.5 ? 0 | (isPositive ? 1 | -1));
        }
    }

    //
    // Modular arithmetic
    //

    /// # Summary
    /// Computes the canonical residue of `value` modulo `modulus`.
    /// The result is always in the range 0..modulus-1 even for negative numbers.
    function ModulusI(value : Int, modulus : Int) : Int {
        Fact(modulus > 0, "`modulus` must be positive");
        let r = value % modulus;
        return (r < 0) ? (r + modulus) | r;
    }

    /// # Summary
    /// Computes the canonical residue of `value` modulo `modulus`.
    /// The result is always in the range 0..modulus-1 even for negative numbers.
    function ModulusL(value : BigInt, modulus : BigInt) : BigInt {
        Fact(modulus > 0L, "`modulus` must be positive");
        let r = value % modulus;
        return (r < 0L) ? (r + modulus) | r;
    }

    /// # Summary
    /// Returns an integer raised to a given power, with respect to a given
    /// modulus. I.e. (expBase^power) % modulus.
    function ExpModI(expBase : Int, power : Int, modulus : Int) : Int {
        Fact(power >= 0, "`power` must be non-negative");
        Fact(modulus > 0, "`modulus` must be positive");
        Fact(expBase > 0, "`expBase` must be positive");

        // shortcut when modulus is 1
        if modulus == 1 {
            return 0;
        }

        mutable res = 1;
        mutable expPow2mod = expBase % modulus;
        mutable powerBits = power;

        while powerBits > 0 {
            if (powerBits &&& 1) != 0 {
                // if bit pₖ is 1, multiply res by expBase^(2^k) (mod `modulus`)
                set res = (res * expPow2mod) % modulus;
            }

            // update value of expBase^(2^k) (mod `modulus`)
            set expPow2mod = (expPow2mod * expPow2mod) % modulus;
            set powerBits >>>= 1;
        }

        return res;
    }

    /// # Summary
    /// Returns an integer raised to a given power, with respect to a given
    /// modulus. I.e. (expBase^power) % modulus.
    function ExpModL(expBase : BigInt, power : BigInt, modulus : BigInt) : BigInt {
        Fact(power >= 0L, "`power` must be non-negative");
        Fact(modulus > 0L, "`modulus` must be positive");
        Fact(expBase > 0L, "`expBase` must be positive");

        // shortcut when modulus is 1
        if modulus == 1L {
            return 0L;
        }

        mutable res = 1L;
        mutable expPow2mod = expBase % modulus;
        mutable powerBits = power;

        while powerBits > 0L {
            if (powerBits &&& 1L) != 0L {
                // if bit pₖ is 1, multiply res by expBase^(2ᵏ) (mod `modulus`)
                set res = (res * expPow2mod) % modulus;
            }

            // update value of expBase^(2ᵏ) (mod `modulus`)
            set expPow2mod = (expPow2mod * expPow2mod) % modulus;
            set powerBits >>>= 1;
        }

        return res;
    }

    /// # Summary
    /// Returns the multiplicative inverse of a modular integer.
    ///
    /// # Description
    /// This will calculate the multiplicative inverse of a
    /// modular integer `b` such that `a • b = 1 (mod modulus)`.
    function InverseModI(a : Int, modulus : Int) : Int {
        let (u, v) = ExtendedGreatestCommonDivisorI(a, modulus);
        let gcd = u * a + v * modulus;
        Fact(gcd == 1, "`a` and `modulus` must be co-prime");
        return ModulusI(u, modulus);
    }

    /// # Summary
    /// Returns the multiplicative inverse of a modular integer.
    ///
    /// # Description
    /// This will calculate the multiplicative inverse of a
    /// modular integer `b` such that `a • b = 1 (mod modulus)`.
    function InverseModL(a : BigInt, modulus : BigInt) : BigInt {
        let (u, v) = ExtendedGreatestCommonDivisorL(a, modulus);
        let gcd = u * a + v * modulus;
        Fact(gcd == 1L, "`a` and `modulus` must be co-prime");
        return ModulusL(u, modulus);
    }

    //
    // GCD, etc.
    //

    /// # Summary
    /// Computes the greatest common divisor of two integers.
    /// Note: GCD is always positive except that GCD(0,0)=0.
    function GreatestCommonDivisorI(a : Int, b : Int) : Int {
        mutable aa = AbsI(a);
        mutable bb = AbsI(b);
        while bb != 0 {
            let cc = aa % bb;
            set aa = bb;
            set bb = cc;
        }
        return aa;
    }

    /// # Summary
    /// Computes the greatest common divisor of two integers.
    /// Note: GCD is always positive except that GCD(0,0)=0.
    function GreatestCommonDivisorL(a : BigInt, b : BigInt) : BigInt {
        mutable aa = AbsL(a);
        mutable bb = AbsL(b);
        while bb != 0L {
            let cc = aa % bb;
            set aa = bb;
            set bb = cc;
        }
        return aa;
    }

    /// # Summary
    /// Returns a tuple (u,v) such that u*a+v*b=GCD(a,b)
    /// Note: GCD is always positive except that GCD(0,0)=0.
    function ExtendedGreatestCommonDivisorI(a : Int, b : Int) : (Int, Int) {
        let signA = SignI(a);
        let signB = SignI(b);
        mutable (s1, s2) = (1, 0);
        mutable (t1, t2) = (0, 1);
        mutable (r1, r2) = (a * signA, b * signB);

        while r2 != 0 {
            let quotient = r1 / r2;
            set (r1, r2) = (r2, r1 - quotient * r2);
            set (s1, s2) = (s2, s1 - quotient * s2);
            set (t1, t2) = (t2, t1 - quotient * t2);
        }

        return (s1 * signA, t1 * signB);
    }

    /// # Summary
    /// Returns a tuple (u,v) such that u*a+v*b=GCD(a,b)
    /// Note: GCD is always positive except that GCD(0,0)=0.
    function ExtendedGreatestCommonDivisorL(a : BigInt, b : BigInt) : (BigInt, BigInt) {
        open Microsoft.Quantum.Convert;
        let signA = IntAsBigInt(SignL(a));
        let signB = IntAsBigInt(SignL(b));
        mutable (s1, s2) = (1L, 0L);
        mutable (t1, t2) = (0L, 1L);
        mutable (r1, r2) = (a * signA, b * signB);

        while r2 != 0L {
            let quotient = r1 / r2;
            set (r1, r2) = (r2, r1 - quotient * r2);
            set (s1, s2) = (s2, s1 - quotient * s2);
            set (t1, t2) = (t2, t1 - quotient * t2);
        }

        return (s1 * signA, t1 * signB);
    }

    /// # Summary
    /// Finds the continued fraction convergent closest to `fraction`
    /// with the denominator less or equal to `denominatorBound`
    /// Using process similar to this: https://nrich.maths.org/1397
    function ContinuedFractionConvergentI(fraction : (Int, Int), denominatorBound : Int) : (Int, Int) {
        Fact(denominatorBound > 0, "Denominator bound must be positive");

        let (a, b) = fraction;
        let signA = SignI(a);
        let signB = SignI(b);
        mutable (s1, s2) = (1, 0);
        mutable (t1, t2) = (0, 1);
        mutable (r1, r2) = (a * signA, b * signB);

        while r2 != 0 and AbsI(s2) <= denominatorBound {
            let quotient = r1 / r2;
            set (r1, r2) = (r2, r1 - quotient * r2);
            set (s1, s2) = (s2, s1 - quotient * s2);
            set (t1, t2) = (t2, t1 - quotient * t2);
        }

        return (r2 == 0 and AbsI(s2) <= denominatorBound)
                ? (-t2 * signB, s2 * signA)
                | (-t1 * signB, s1 * signA);
    }

    //
    // Binary, bits, etc.
    //

    /// # Summary
    /// For a non-negative integer `a`, returns the number of bits required to represent `a`.
    /// NOTE: This function returns the smallest n such that a < 2^n.
    function BitSizeI(a : Int) : Int {
        Fact(a >= 0, "`a` must be non-negative.");
        mutable number = a;
        mutable size = 0;
        while (number != 0) {
            set size = size + 1;
            set number = number >>> 1;
        }
        return size;
    }

    //
    // Complex numbers
    //

    /// # Summary
    /// Represents a complex number by its real and imaginary components.
    /// The first element of the tuple is the real component,
    /// the second one - the imaginary component.
    ///
    /// # Example
    /// The following snippet defines the imaginary unit 𝑖 = 0 + 1𝑖:
    /// ```qsharp
    /// let imagUnit = Complex(0.0, 1.0);
    /// ```
    newtype Complex = (Real: Double, Imag: Double);

    /// # Summary
    /// Represents a complex number in polar form.
    /// The polar representation of a complex number is c = r⋅𝑒^(t𝑖).
    ///
    /// # Named Items
    /// ## Magnitude
    /// The absolute value r>0 of c.
    /// ## Argument
    /// The phase t ∈ ℝ of c.
    newtype ComplexPolar = (Magnitude: Double, Argument: Double);

    /// # Summary
    /// Returns the squared absolute value of a complex number of type
    /// `Complex`.
    ///
    /// # Input
    /// ## input
    /// Complex number c = x + y𝑖.
    ///
    /// # Output
    /// Squared absolute value |c|² = x² + y².
    function AbsSquaredComplex(input : Complex) : Double {
        return input::Real * input::Real + input::Imag * input::Imag;
    }

    /// # Summary
    /// Returns the absolute value of a complex number of type
    /// `Complex`.
    ///
    /// # Input
    /// ## input
    /// Complex number c = x + y𝑖.
    ///
    /// # Output
    /// Absolute value |c| = √(x² + y²).
    function AbsComplex(input : Complex) : Double {
        Sqrt(AbsSquaredComplex(input))
    }

    /// # Summary
    /// Returns the phase of a complex number of type
    /// `Complex`.
    ///
    /// # Input
    /// ## input
    /// Complex number c = x + y𝑖.
    ///
    /// # Output
    /// Phase Arg(c) = ArcTan(y,x) ∈ (-𝜋,𝜋].
    function ArgComplex(input : Complex) : Double {
        ArcTan2(input::Imag, input::Real)
    }

    /// # Summary
    /// Returns the squared absolute value of a complex number of type
    /// `ComplexPolar`.
    ///
    /// # Input
    /// ## input
    /// Complex number c = r⋅𝑒^(t𝑖).
    ///
    /// # Output
    /// Squared absolute value |c|² = r².
    function AbsSquaredComplexPolar(input : ComplexPolar) : Double {
        input::Magnitude * input::Magnitude
    }

    /// # Summary
    /// Returns the absolute value of a complex number of type
    /// `ComplexPolar`.
    ///
    /// # Input
    /// ## input
    /// Complex number c = r⋅𝑒^(t𝑖).
    ///
    /// # Output
    /// Absolute value |c| = r.
    function AbsComplexPolar(input : ComplexPolar) : Double {
        input::Magnitude
    }

    /// # Summary
    /// Returns the phase of a complex number of type `ComplexPolar`.
    ///
    /// # Input
    /// ## input
    /// Complex number c = r⋅𝑒^(t𝑖).
    ///
    /// # Output
    /// Phase Arg(c) = t.
    function ArgComplexPolar(input : ComplexPolar) : Double {
        input::Argument
    }

    /// # Summary
    /// Returns the unary negation of an input of type `Complex`.
    ///
    /// # Input
    /// ## input
    /// A value whose negation is to be returned.
    ///
    /// # Output
    /// The unary negation of `input`.
    function NegationC(input : Complex) : Complex {
        Complex(-input::Real, -input::Imag)
    }

    /// # Summary
    /// Returns the unary negation of an input of type `ComplexPolar`
    ///
    /// # Input
    /// ## input
    /// A value whose negation is to be returned.
    ///
    /// # Output
    /// The unary negation of `input`.
    function NegationCP(input : ComplexPolar) : ComplexPolar {
        return ComplexPolar(input::Magnitude, input::Argument + PI());
    }

    /// # Summary
    /// Returns the sum of two inputs of type `Complex`.
    ///
    /// # Input
    /// ## a
    /// The first input a to be summed.
    /// ## b
    /// The second input b to be summed.
    ///
    /// # Output
    /// The sum a + b.
    function PlusC(a : Complex, b : Complex) : Complex {
        Complex(a::Real + b::Real, a::Imag + b::Imag)
    }

    /// # Summary
    /// Returns the sum of two inputs of type `ComplexPolar`.
    ///
    /// # Input
    /// ## a
    /// The first input a to be summed.
    /// ## b
    /// The second input b to be summed.
    ///
    /// # Output
    /// The sum a + b.
    function PlusCP(a : ComplexPolar, b : ComplexPolar) : ComplexPolar {
        open Microsoft.Quantum.Convert;
        return ComplexAsComplexPolar(
            PlusC(
                ComplexPolarAsComplex(a),
                ComplexPolarAsComplex(b)
            )
        );
    }

    /// # Summary
    /// Returns the difference between two inputs of type `Complex`.
    ///
    /// # Input
    /// ## a
    /// The first input a to be subtracted.
    /// ## b
    /// The second input b to be subtracted.
    ///
    /// # Output
    /// The difference a - b.
    function MinusC(a : Complex, b : Complex) : Complex {
        Complex(a::Real - b::Real, a::Imag - b::Imag)
    }

    /// # Summary
    /// Returns the difference between two inputs of type `ComplexPolar`.
    ///
    /// # Input
    /// ## a
    /// The first input a to be subtracted.
    /// ## b
    /// The second input b to be subtracted.
    ///
    /// # Output
    /// The difference a - b.
    function MinusCP(a : ComplexPolar, b : ComplexPolar) : ComplexPolar {
        PlusCP(a, NegationCP(b))
    }

    /// # Summary
    /// Returns the product of two inputs of type `Complex`.
    ///
    /// # Input
    /// ## a
    /// The first input a to be multiplied.
    /// ## b
    /// The second input b to be multiplied.
    ///
    /// # Output
    /// The product a⋅b.
    function TimesC(a : Complex, b : Complex) : Complex {
        Complex(
            a::Real * b::Real - a::Imag * b::Imag,
            a::Real * b::Imag + a::Imag * b::Real
        )
    }

    /// # Summary
    /// Returns the product of two inputs of type `ComplexPolar`.
    ///
    /// # Input
    /// ## a
    /// The first input a to be multiplied.
    /// ## b
    /// The second input b to be multiplied.
    ///
    /// # Output
    /// The product a⋅b.
    function TimesCP(a : ComplexPolar, b : ComplexPolar) : ComplexPolar {
        ComplexPolar(
            a::Magnitude * b::Magnitude,
            a::Argument + b::Argument
        )
    }

    /// # Summary
    /// Internal. Since it is easiest to define the power of two complex numbers
    /// in cartesian form as returning in polar form, we define that here, then
    /// convert as needed.
    /// Note that this is a multi-valued function, but only one value is returned.
    internal function PowCAsCP(base : Complex, power : Complex) : ComplexPolar {
        let ((a, b), (c, d)) = (base!, power!);
        let baseSqNorm = a*a + b*b;
        let baseNorm = Sqrt(baseSqNorm);
        let baseArg = ArgComplex(base);

        // We pick the principal value of the multi-valued complex function ㏑ as
        // ㏑(a+b𝑖) = ln(|a+b𝑖|) + 𝑖⋅arg(a+b𝑖) = ln(baseNorm) + 𝑖⋅baseArg
        // Therefore
        // base^power = (a+b𝑖)^(c+d𝑖) = 𝑒^( (c+d𝑖)⋅㏑(a+b𝑖) ) =
        // = 𝑒^( (c+d𝑖)⋅(ln(baseNorm)+𝑖⋅baseArg) ) = 
        // = e^( (c⋅ln(baseNorm) - d⋅baseArg) + 𝑖⋅(c⋅baseArg + d⋅ln(baseNorm)) )
        // magnitude = e^((c⋅ln(baseNorm) - d⋅baseArg)) = baseNorm^c / e^(d⋅baseArg)
        // angle = d⋅ln(baseNorm) + c⋅baseArg

        let magnitude = baseNorm^c / E()^(d * baseArg);
        let angle = d * Log(baseNorm) + c * baseArg;

        ComplexPolar(magnitude, angle)
    }

    /// # Summary
    /// Returns a number raised to a given power of type `Complex`.
    /// Note that this is a multi-valued function, but only one value is returned.
    ///
    /// # Input
    /// ## a
    /// The number a that is to be raised.
    /// ## power
    /// The power b to which a should be raised.
    ///
    /// # Output
    /// The power a^b
    function PowC(a : Complex, power : Complex) : Complex {
        open Microsoft.Quantum.Convert;
        ComplexPolarAsComplex(PowCAsCP(a, power))
    }

    /// # Summary
    /// Returns a number raised to a given power of type `ComplexPolar`.
    /// Note that this is a multi-valued function, but only one value is returned.
    ///
    /// # Input
    /// ## a
    /// The number a that is to be raised.
    /// ## power
    /// The power b to which a should be raised.
    ///
    /// # Output
    /// The power a^b
    function PowCP(a : ComplexPolar, power : ComplexPolar) : ComplexPolar {
        open Microsoft.Quantum.Convert;
        PowCAsCP(ComplexPolarAsComplex(a), ComplexPolarAsComplex(power))
    }

    /// # Summary
    /// Returns the quotient of two inputs of type `Complex`.
    ///
    /// # Input
    /// ## a
    /// The first input a to be divided.
    /// ## b
    /// The second input b to be divided.
    ///
    /// # Output
    /// The quotient a / b.
    function DividedByC(a : Complex, b : Complex) : Complex {
        let sqNorm = b::Real * b::Real + b::Imag * b::Imag;
        Complex(
            (a::Real * b::Real + a::Imag * b::Imag) / sqNorm,
            (a::Imag * b::Real - a::Real * b::Imag) / sqNorm
        )
    }

    /// # Summary
    /// Returns the quotient of two inputs of type `ComplexPolar`.
    ///
    /// # Input
    /// ## a
    /// The first input a to be divided.
    /// ## b
    /// The second input b to be divided.
    ///
    /// # Output
    /// The quotient a / b.
    function DividedByCP(a : ComplexPolar, b : ComplexPolar) : ComplexPolar {
        ComplexPolar(a::Magnitude / b::Magnitude, a::Argument - b::Argument)
    }

}
