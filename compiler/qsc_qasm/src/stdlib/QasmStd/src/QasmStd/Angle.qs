// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// This file defines the Angle type and its associated functions.
/// It is an internal implementation detail for OpenQASM compilation
/// and is not intended for use outside of this context.

import Std.Arrays.Reversed;
import Std.Convert.BigIntAsBoolArray;
import Std.Convert.BoolArrayAsInt;
import Std.Convert.IntAsBigInt;
import Std.Convert.IntAsBoolArray;
import Std.Convert.IntAsDouble;
import Std.Diagnostics.Fact;
import Std.Math.RoundHalfAwayFromZero;

// Export the Angle type and its associated functions.
export Angle;
// Export the array conversion functions for Angle.
export AngleAsBoolArrayBE, AngleAsResultArray;
// Export cast from Angle to other types.
export AngleAsDouble, AngleAsBool, AngleAsResult;
// Export cast from other types to Angle.
export IntAsAngle, DoubleAsAngle, ResultAsAngle;
// Export width conversion functions for Angle.
export AdjustAngleSizeNoTruncation;
// Export bitwise operations on Angle.
export AngleShl, AngleShr, AngleNotB, AngleAndB, AngleOrB, AngleXorB;
// Export comparison functions for Angle.
export AngleEq, AngleNeq, AngleGt, AngleGte, AngleLt, AngleLte;
// Export symmetric functions.
export AddAngles, SubtractAngles, DivideAngleByAngle, NegAngle;
// Export asymmetric functions.
export MultiplyAngleByInt, MultiplyAngleByBigInt, DivideAngleByInt;


struct Angle {
    Value : Int,
    Size : Int
}

function AngleAsBoolArrayBE(angle : Angle) : Bool[] {
    Reversed(IntAsBoolArray(angle.Value, angle.Size))
}

function AngleAsResultArray(angle : Angle) : Result[] {
    Convert.IntAsResultArrayBE(angle.Value, angle.Size)
}

function AngleAsDouble(angle : Angle) : Double {
    let F64_MANTISSA_DIGITS = 53;
    let angle = if angle.Size > F64_MANTISSA_DIGITS {
        AdjustAngleSize(angle, F64_MANTISSA_DIGITS, false)
    } else {
        angle
    };
    let denom = IntAsDouble(1 <<< angle.Size);
    let value = IntAsDouble(angle.Value);
    let factor = (2.0 * Std.Math.PI()) / denom;
    value * factor
}

function AngleAsBool(angle : Angle) : Bool {
    return angle.Value != 0;
}

function ResultAsAngle(result : Result) : Angle {
    new Angle { Value = Convert.ResultAsInt(result), Size = 1 }
}

function AngleAsResult(angle : Angle) : Result {
    Std.Convert.BoolAsResult(angle.Value != 0)
}

function IntAsAngle(value : Int, size : Int) : Angle {
    Fact(value >= 0, "Value must be >= 0");
    Fact(size > 0, "Size must be > 0");
    new Angle { Value = value, Size = size }
}

function DoubleAsAngle(value : Double, size : Int) : Angle {
    let tau : Double = 2. * Std.Math.PI();

    mutable value = value % tau;
    if value < 0. {
        value = value + tau;
    }

    Fact(value >= 0., "Value must be >= 0.");
    Fact(value < tau, "Value must be < tau.");
    Fact(size > 0, "Size must be > 0");


    let factor = tau / Std.Convert.IntAsDouble(1 <<< size);
    let value = RoundHalfAwayFromZero(value / factor);
    new Angle { Value = value, Size = size }
}

function AdjustAngleSizeNoTruncation(angle : Angle, new_size : Int) : Angle {
    AdjustAngleSize(angle, new_size, false)
}

function AdjustAngleSize(angle : Angle, new_size : Int, truncate : Bool) : Angle {
    Fact(new_size > 0, "New size must be > 0");
    let (value, size) = (angle.Value, angle.Size);
    if new_size < size {
        let value = if truncate {
            let shift_amount = size - new_size;
            value >>> shift_amount
        } else {
            // Rounding
            let shift_amount = size - new_size;
            let half = 1 <<< (shift_amount - 1);
            let mask = (1 <<< shift_amount) - 1;
            let lower_bits = value &&& mask;
            let upper_bits = value >>> shift_amount;
            if lower_bits > half or (lower_bits == half and (upper_bits &&& 1) == 1) {
                upper_bits + 1
            } else {
                upper_bits
            }
        };
        new Angle { Value = value, Size = size }
    } elif new_size == size {
        // Same size, no change
        angle
    } else {
        // Padding with zeros
        let value = value <<< (new_size - size);
        new Angle { Value = value, Size = size }
    }
}

// Bit shift

function AngleShl(angle : Angle, operand : Int) : Angle {
    Fact(operand >= 0, "Shift amount must be >= 0");
    let mask = (1 <<< angle.Size) - 1;
    let value = (angle.Value <<< operand) &&& mask;
    new Angle { Value = value, Size = angle.Size }
}

function AngleShr(angle : Angle, operand : Int) : Angle {
    Fact(operand >= 0, "Shift amount must be >= 0");
    let value = (angle.Value >>> operand);
    new Angle { Value = value, Size = angle.Size }
}

// Bitwise

function AngleNotB(angle : Angle) : Angle {
    let mask = (1 <<< angle.Size) - 1;
    let value = (~~~angle.Value) &&& mask;
    new Angle { Value = value, Size = angle.Size }
}

function AngleAndB(lhs : Angle, rhs : Angle) : Angle {
    Fact(lhs.Size == rhs.Size, "Angle sizes must be the same");
    let value = lhs.Value &&& rhs.Value;
    new Angle { Value = value, Size = lhs.Size }
}

function AngleOrB(lhs : Angle, rhs : Angle) : Angle {
    Fact(lhs.Size == rhs.Size, "Angle sizes must be the same");
    let value = lhs.Value ||| rhs.Value;
    new Angle { Value = value, Size = lhs.Size }
}

function AngleXorB(lhs : Angle, rhs : Angle) : Angle {
    Fact(lhs.Size == rhs.Size, "Angle sizes must be the same");
    let value = lhs.Value ^^^ rhs.Value;
    new Angle { Value = value, Size = lhs.Size }
}

// Comparison

function AngleEq(lhs : Angle, rhs : Angle) : Bool {
    Fact(lhs.Size == rhs.Size, "Angle sizes must be the same");
    lhs.Value == rhs.Value
}

function AngleNeq(lhs : Angle, rhs : Angle) : Bool {
    Fact(lhs.Size == rhs.Size, "Angle sizes must be the same");
    lhs.Value != rhs.Value
}

function AngleGt(lhs : Angle, rhs : Angle) : Bool {
    Fact(lhs.Size == rhs.Size, "Angle sizes must be the same");
    lhs.Value > rhs.Value
}

function AngleGte(lhs : Angle, rhs : Angle) : Bool {
    Fact(lhs.Size == rhs.Size, "Angle sizes must be the same");
    lhs.Value >= rhs.Value
}

function AngleLt(lhs : Angle, rhs : Angle) : Bool {
    Fact(lhs.Size == rhs.Size, "Angle sizes must be the same");
    lhs.Value < rhs.Value
}

function AngleLte(lhs : Angle, rhs : Angle) : Bool {
    Fact(lhs.Size == rhs.Size, "Angle sizes must be the same");
    lhs.Value <= rhs.Value
}

// Arithmetic

function AddAngles(lhs : Angle, rhs : Angle) : Angle {
    Fact(lhs.Size == rhs.Size, "Angle sizes must be the same");
    let value = (lhs.Value + rhs.Value) % (1 <<< lhs.Size);
    new Angle { Value = value, Size = lhs.Size }
}

function SubtractAngles(lhs : Angle, rhs : Angle) : Angle {
    Fact(lhs.Size == rhs.Size, "Angle sizes must be the same");
    let value = (lhs.Value + ((1 <<< lhs.Size) -  rhs.Value)) % (1 <<< lhs.Size);
    new Angle { Value = value, Size = lhs.Size }
}

function MultiplyAngleByInt(angle : Angle, factor : Int) : Angle {
    Fact(factor >= 0, "Factor amount must be >= 0");
    let value = (angle.Value * factor) % (1 <<< angle.Size);
    new Angle { Value = value, Size = angle.Size }
}

function MultiplyAngleByBigInt(angle : Angle, factor : BigInt) : Angle {
    Fact(factor >= 0L, "Factor amount must be >= 0");
    let value : BigInt = Std.Convert.IntAsBigInt(angle.Value);
    let value = (value * factor) % Std.Convert.IntAsBigInt(1 <<< angle.Size);
    let value = Std.Convert.BigIntAsInt(value);
    new Angle { Value = value, Size = angle.Size }
}

function DivideAngleByAngle(lhs : Angle, rhs : Angle) : Int {
    Fact(lhs.Size == rhs.Size, "Angle sizes must be the same");
    let value = lhs.Value / rhs.Value;
    value
}

function DivideAngleByInt(angle : Angle, divisor : Int) : Angle {
    Fact(divisor > 0, "Divisor amount must be > 0");
    let value = angle.Value / divisor;
    new Angle { Value = value, Size = angle.Size }
}

function NegAngle(angle : Angle) : Angle {
    let (value, size) = (angle.Value, angle.Size);
    let value = ((1 <<< size) - value) % (1 <<< size);
    new Angle { Value = value, Size = size }
}
