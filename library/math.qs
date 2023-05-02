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
        if   (a < 0) { return -1; }
        elif (a > 0) { return +1; }
        else         { return  0; }
    }
    
    /// # Summary
    /// Returns -1, 0 or +1 that indicates the sign of a number.
    function SignD (a : Double) : Int {
        if   (a < 0.0) { return -1; }
        elif (a > 0.0) { return +1; }
        else           { return  0; }
    }

    /// # Summary
    /// Returns -1, 0 or +1 that indicates the sign of a number.
    function SignL (a : BigInt) : Int {
        if   (a < 0L) { return -1; }
        elif (a > 0L) { return +1; }
        else          { return  0; }
    }

    /// # Summary
    /// Returns the absolute value of an integer.
    function AbsI (a : Int) : Int {
        return a < 0 ? -a | a;
    }
    
    /// # Summary
    /// Returns the absolute value of a double-precision floating-point number.
    function AbsD (a : Double) : Double {
        return a < 0.0 ? -a | a;
    }

    /// # Summary
    function AbsL (a : BigInt) : BigInt {
        return a < 0L ? -a | a;
    }
    
    /// # Summary
    /// Returns the larger of two specified numbers.
    function MaxI(a : Int, b : Int) : Int {
        return a > b ? a | b;
    }

    /// # Summary
    /// Returns the larger of two specified numbers.
    function MaxD(a : Double, b : Double) : Double {
        return a > b ? a | b;
    }

    /// # Summary
    /// Returns the larger of two specified numbers.
    function MaxL (a : BigInt, b : BigInt) : BigInt {
        return a > b ? a | b;
    }

    /// # Summary
    /// Returns the smaller of two specified numbers.
    function MinI (a : Int, b : Int) : Int {
        return a < b ? a | b;
    }

    /// # Summary
    /// Returns the smaller of two specified numbers.
    function MinD (a : Double, b : Double) : Double {
        return a < b ? a | b;
    }

    /// # Summary
    /// Returns the smaller of two specified numbers.
    function MinL(a : BigInt, b : BigInt) : BigInt {
        return a < b ? a | b;
    }

    /// # Summary
    /// Given an array of integers, returns the largest element.
    function Max(values : Int[]) : Int {
        mutable max = values[0];
        for idx in 1 .. values::Length-1 {
            if values[idx] > max {
                set max = values[idx];
            }
        }
        return max;
    }

    /// # Summary
    /// Given an array of integers, returns the smallest element.
    function Min(values : Int[]) : Int {
        mutable min = values[0];
        for idx in 1 .. values::Length-1 {
            if values[idx] < min {
                set min = values[idx];
            }
        }
        return min;
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
        return Log(x + Sqrt(x * x - 1.0));
    }

    /// # Summary
    /// Computes the inverse hyperbolic sine of a number.
    function ArcSinh (x : Double) : Double {
        return Log(x + Sqrt(x * x + 1.0));
    }


    /// # Summary
    /// Computes the inverse hyperbolic tangent of a number.
    function ArcTanh (x : Double) : Double {
        return Log((1.0 + x) / (1.0 - x)) * 0.5;
    }

    //
    // Log, exp, etc.
    //

    /// # Summary
    /// Returns the square root of a specified number.
    function Sqrt(d : Double) : Double {
        body intrinsic;
    }

    /// # Summary
    /// Returns the natural (base $e$) logarithm of a specified number.
    function Log(input : Double) : Double {
        body intrinsic;
    }

    /// # Summary
    /// Returns the base-10 logarithm of a specified number.
    function Log10(input : Double) : Double {
        return Log(input) / Log(10.0);
    }

    /// # Summary
    /// Computes the base-2 logarithm of a number.
    function Lg(input : Double) : Double {
        return Log(input) / Log(2.0);
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
    // TODO: reconcile between ModPowL, ExpModI and ExpModL
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
    /// Performs modular division on a number raised to the power of another number.
    /// I.e. (value^exponent) % modulus
    // TODO: Do we need this at all?
    function ModPowL(value : BigInt, exponent : BigInt, modulus : BigInt) : BigInt {
        return ExpModL(value, exponent, modulus);
    }

    //
    // Binary, bits, etc.
    //

    /// # Summary
    /// For a non-negative integer `a`, returns the number of bits required to represent `a`.
    ///
    /// # Remarks
    /// This function returns the smallest $n$ such that $a < 2^n$.
    ///
    /// # Input
    /// ## a
    /// The integer whose bit-size is to be computed.
    ///
    /// # Output
    /// The bit-size of `a`.
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
