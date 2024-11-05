// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import Std.Convert.IntAsDouble, Std.Convert.BoolArrayAsInt;
import Std.Math.AbsD, Std.Math.Floor;
import Std.Arrays.Most, Std.Arrays.Tail;

/// # Summary
/// Computes fixed-point approximation for a double and returns it as `Bool` array.
///
/// # Input
/// ## integerBits
/// Assumed number of integer bits (including the sign bit).
/// ## fractionalBits
/// Assumed number of fractional bits.
/// ## value
/// Value to be approximated.
///
/// # Example
/// Note that the first element in the Boolean array is the least-significant bit.
/// ```qsharp
/// let bits = FixedPointAsBoolArray(2, 2,  1.25); // bits = [true, false, true, false]
/// let bits = FixedPointAsBoolArray(2, 2,  1.3);  // bits = [true, false, true, false], approximated
/// let bits = FixedPointAsBoolArray(2, 2, -1.75); // bits = [true, false, false, true], two's complement
/// ```
function FixedPointAsBoolArray(integerBits : Int, fractionalBits : Int, value : Double) : Bool[] {
    let numBits = integerBits + fractionalBits;
    let sign = value < 0.0;

    mutable result = [false, size = numBits];
    mutable rescaledConstant = 2.0^IntAsDouble(fractionalBits) * AbsD(value) + 0.5;
    mutable keepAdding = sign;

    for idx in 0..numBits - 1 {
        let intConstant = Floor(rescaledConstant);
        set rescaledConstant = rescaledConstant / 2.0;
        mutable currentBit = (intConstant &&& 1) == (sign ? 0 | 1);
        if keepAdding {
            set keepAdding = currentBit;
            set currentBit = not currentBit;
        }
        if currentBit {
            set result w/= idx <- true;
        }
    }

    return result;
}

/// # Summary
/// Returns the double value of a fixed-point approximation from of a `Bool` array.
///
/// # Input
/// ## integerBits
/// Assumed number of integer bits (including the sign bit).
/// ## bits
/// Bit-string representation of approximated number.
///
/// # Example
/// Note that the first element in the Boolean array is the least-significant bit.
/// ```qsharp
/// let value = BoolArrayAsFixedPoint(2, [true, false, true, false]); // value =  1.25
/// let value = BoolArrayAsFixedPoint(2, [true, false, false, true]); // value = -1.75
/// ```

function BoolArrayAsFixedPoint(integerBits : Int, bits : Bool[]) : Double {
    let numBits = Length(bits);
    let intPart = (Tail(bits) ? -(1 <<< (numBits - 1)) | 0) + BoolArrayAsInt(Most(bits));
    return IntAsDouble(intPart) / (2.0^IntAsDouble(numBits - integerBits));
}

/// # Summary
/// Discretizes a double value as a fixed-point approximation and returns its value as a double.
///
/// # Input
/// ## integerBits
/// Assumed number of integer bits (including the sign bit).
/// ## fractionalBits
/// Assumed number of fractional bits.
/// ## value
/// Value to be approximated.
///
/// # Example
/// ```qsharp
/// let value = DoubleAsFixedPoint(2, 2, 1.3); // value = 1.25
/// let value = DoubleAsFixedPoint(2, 2, 0.8); // value = 0.75
/// ```
function DoubleAsFixedPoint(integerBits : Int, fractionalBits : Int, value : Double) : Double {
    return BoolArrayAsFixedPoint(integerBits, FixedPointAsBoolArray(integerBits, fractionalBits, value));
}

export
    FixedPointAsBoolArray,
    BoolArrayAsFixedPoint,
    DoubleAsFixedPoint;