// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Math {
    open Microsoft.Quantum.Diagnostics;

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
