// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import Std.Arrays.Reversed;
import Std.Convert.BigIntAsBoolArray;
import Std.Convert.BoolArrayAsInt;
import Std.Convert.IntAsBigInt;
import Std.Convert.IntAsBoolArray;
import Std.Convert.IntAsDouble;
import Std.Diagnostics.Fact;
import Std.Math.RoundHalfAwayFromZero;
import Convert.__IntAsResultArrayBE__;
import Convert.__ResultAsInt__;

export __Angle__, __AngleAsBoolArrayBE__, __AngleAsResultArray__, __AngleAsDouble__, __AngleAsBool__, __AngleAsResult__, __IntAsAngle__, __DoubleAsAngle__, __ConvertAngleToWidth__, __ConvertAngleToWidthNoTrunc__, __AngleShl__, __AngleShr__, __AngleNotB__, __AngleAndB__, __AngleOrB__, __AngleXorB__, __AngleEq__, __AngleNeq__, __AngleGt__, __AngleGte__, __AngleLt__, __AngleLte__, __AddAngles__, __SubtractAngles__, __MultiplyAngleByInt__, __MultiplyAngleByBigInt__, __DivideAngleByInt__, __DivideAngleByAngle__, __NegAngle__, __ResultAsAngle__;


struct __Angle__ {
    Value : Int,
    Size : Int
}

function __AngleAsBoolArrayBE__(angle : __Angle__) : Bool[] {
    Reversed(IntAsBoolArray(angle.Value, angle.Size))
}

function __AngleAsResultArray__(angle : __Angle__) : Result[] {
    let (number, bits) = angle!;
    __IntAsResultArrayBE__(number, bits)
}

function __AngleAsDouble__(angle : __Angle__) : Double {
    let F64_MANTISSA_DIGITS = 53;
    let (value, size) = if angle.Size > F64_MANTISSA_DIGITS {
        __ConvertAngleToWidth__(angle, F64_MANTISSA_DIGITS, false)!
    } else {
        angle!
    };
    let denom = IntAsDouble(1 <<< size);
    let value = IntAsDouble(value);
    let factor = (2.0 * Std.Math.PI()) / denom;
    value * factor
}

function __AngleAsBool__(angle : __Angle__) : Bool {
    return angle.Value != 0;
}

function __ResultAsAngle__(result: Result) : __Angle__ {
    new __Angle__ { Value = __ResultAsInt__(result), Size = 1 }
}

function __AngleAsResult__(angle : __Angle__) : Result {
    Microsoft.Quantum.Convert.BoolAsResult(angle.Value != 0)
}

function __IntAsAngle__(value : Int, size : Int) : __Angle__ {
    Fact(value >= 0, "Value must be >= 0");
    Fact(size > 0, "Size must be > 0");
    new __Angle__ { Value = value, Size = size }
}

function __DoubleAsAngle__(value : Double, size : Int) : __Angle__ {
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
    new __Angle__ { Value = value, Size = size }
}

function __ConvertAngleToWidthNoTrunc__(angle : __Angle__, new_size : Int) : __Angle__ {
    __ConvertAngleToWidth__(angle, new_size, false)
}

function __ConvertAngleToWidth__(angle : __Angle__, new_size : Int, truncate : Bool) : __Angle__ {
    let (value, size) = angle!;
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
        new __Angle__ { Value = value, Size = size }
    } elif new_size == size {
        // Same size, no change
        angle
    } else {
        // Padding with zeros
        let value = value <<< (new_size - size);
        new __Angle__ { Value = value, Size = size }
    }
}

// Bit shift

function __AngleShl__(lhs : __Angle__, rhs : Int) : __Angle__ {
    let (lhs_value, lhs_size) = lhs!;
    let mask = (1 <<< lhs_size) - 1;
    let value = (lhs_value <<< rhs) &&& mask;
    new __Angle__ { Value = value, Size = lhs_size }
}

function __AngleShr__(lhs : __Angle__, rhs : Int) : __Angle__ {
    let (lhs_value, lhs_size) = lhs!;
    let value = (lhs_value >>> rhs);
    new __Angle__ { Value = value, Size = lhs_size }
}

// Bitwise

function __AngleNotB__(angle : __Angle__) : __Angle__ {
    let (value, size) = angle!;
    let mask = (1 <<< size) - 1;
    let value = (~~~value) &&& mask;
    new __Angle__ { Value = value, Size = size }
}

function __AngleAndB__(lhs : __Angle__, rhs : __Angle__) : __Angle__ {
    let (lhs_value, lhs_size) = lhs!;
    let (rhs_value, rhs_size) = rhs!;
    Fact(lhs_size == rhs_size, "Angle sizes must be the same");
    let value = lhs_value &&& rhs_value;
    new __Angle__ { Value = value, Size = lhs_size }
}

function __AngleOrB__(lhs : __Angle__, rhs : __Angle__) : __Angle__ {
    let (lhs_value, lhs_size) = lhs!;
    let (rhs_value, rhs_size) = rhs!;
    Fact(lhs_size == rhs_size, "Angle sizes must be the same");
    let value = lhs_value ||| rhs_value;
    new __Angle__ { Value = value, Size = lhs_size }
}

function __AngleXorB__(lhs : __Angle__, rhs : __Angle__) : __Angle__ {
    let (lhs_value, lhs_size) = lhs!;
    let (rhs_value, rhs_size) = rhs!;
    Fact(lhs_size == rhs_size, "Angle sizes must be the same");
    let value = lhs_value ^^^ rhs_value;
    new __Angle__ { Value = value, Size = lhs_size }
}

// Comparison

function __AngleEq__(lhs: __Angle__, rhs: __Angle__) : Bool {
    let (lhs_value, lhs_size) = lhs!;
    let (rhs_value, rhs_size) = rhs!;
    Fact(lhs_size == rhs_size, "Angle sizes must be the same");
    lhs_value == rhs_value
}

function __AngleNeq__(lhs: __Angle__, rhs: __Angle__) : Bool {
    let (lhs_value, lhs_size) = lhs!;
    let (rhs_value, rhs_size) = rhs!;
    Fact(lhs_size == rhs_size, "Angle sizes must be the same");
    lhs_value != rhs_value
}

function __AngleGt__(lhs: __Angle__, rhs: __Angle__) : Bool {
    let (lhs_value, lhs_size) = lhs!;
    let (rhs_value, rhs_size) = rhs!;
    Fact(lhs_size == rhs_size, "Angle sizes must be the same");
    lhs_value > rhs_value
}

function __AngleGte__(lhs: __Angle__, rhs: __Angle__) : Bool {
    let (lhs_value, lhs_size) = lhs!;
    let (rhs_value, rhs_size) = rhs!;
    Fact(lhs_size == rhs_size, "Angle sizes must be the same");
    lhs_value >= rhs_value
}

function __AngleLt__(lhs: __Angle__, rhs: __Angle__) : Bool {
    let (lhs_value, lhs_size) = lhs!;
    let (rhs_value, rhs_size) = rhs!;
    Fact(lhs_size == rhs_size, "Angle sizes must be the same");
    lhs_value < rhs_value
}

function __AngleLte__(lhs: __Angle__, rhs: __Angle__) : Bool {
    let (lhs_value, lhs_size) = lhs!;
    let (rhs_value, rhs_size) = rhs!;
    Fact(lhs_size == rhs_size, "Angle sizes must be the same");
    lhs_value <= rhs_value
}

// Arithmetic

function __AddAngles__(lhs : __Angle__, rhs : __Angle__) : __Angle__ {
    let (lhs_value, lhs_size) = lhs!;
    let (rhs_value, rhs_size) = rhs!;
    Fact(lhs_size == rhs_size, "Angle sizes must be the same");
    let value = (lhs_value + rhs_value) % (1 <<< lhs_size);
    new __Angle__ { Value = value, Size = lhs_size }
}

function __SubtractAngles__(lhs : __Angle__, rhs : __Angle__) : __Angle__ {
    let (lhs_value, lhs_size) = lhs!;
    let (rhs_value, rhs_size) = rhs!;
    Fact(lhs_size == rhs_size, "Angle sizes must be the same");
    let value = (lhs_value + ((1 <<< lhs_size) - rhs_value)) % (1 <<< lhs_size);
    new __Angle__ { Value = value, Size = lhs_size }
}

function __MultiplyAngleByInt__(angle : __Angle__, factor : Int) : __Angle__ {
    let (value, size) = angle!;
    let value = (value * factor) % (1 <<< size);
    new __Angle__ { Value = value, Size = size }
}

function __MultiplyAngleByBigInt__(angle : __Angle__, factor : BigInt) : __Angle__ {
    let (value, size) = angle!;
    let value : BigInt = Std.Convert.IntAsBigInt(value);
    let value = (value * factor) % Std.Convert.IntAsBigInt(1 <<< size);
    let value = Std.Convert.BoolArrayAsInt(Std.Convert.BigIntAsBoolArray(value, size));
    new __Angle__ { Value = value, Size = size }
}

function __DivideAngleByAngle__(lhs : __Angle__, rhs : __Angle__) : Int {
    let (lhs_value, lhs_size) = lhs!;
    let (rhs_value, rhs_size) = rhs!;
    Fact(lhs_size == rhs_size, "Angle sizes must be the same");
    let value = lhs_value / rhs_value;
    value
}

function __DivideAngleByInt__(angle : __Angle__, divisor : Int) : __Angle__ {
    let (value, size) = angle!;
    let value = value / divisor;
    new __Angle__ { Value = value, Size = size }
}

function __NegAngle__(angle : __Angle__) : __Angle__ {
    let (value, size) = angle!;
    let value = (1 <<< size) - value;
    new __Angle__ { Value = value, Size = size }
}
