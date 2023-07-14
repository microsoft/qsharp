// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Math {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Convert;

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
    /// to its diameter, $\pi \approx 3.14159265358979323846$.
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
    /// $e \approx 2.7182818284590452354$.
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
    /// modular integer $b$ such that $a \cdot b = 1 (\operatorname{mod} \texttt{modulus})$.
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
    /// modular integer $b$ such that $a \cdot b = 1 (\operatorname{mod} \texttt{modulus})$.
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
    function ContinuedFractionConvergentI(fraction : (Int, Int), denominatorBound : Int): (Int, Int) {
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
}
