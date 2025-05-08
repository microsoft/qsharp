import Std.Math.Truncate;
import Angle.IntAsAngle;
import Angle.Angle;
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// This file defines the type conversion for casting gerneral types.
/// It is an internal implementation detail for OpenQASM compilation
/// and is not intended for use outside of this context.

/// The ``BOOL_AS_RESULT`` function is used to implement the cast expr in QASM for bool to bit.
/// This already exists in the Q# library, but is defined as a marker for casts from QASM.
function BoolAsResult(input : Bool) : Result {
    Std.Convert.BoolAsResult(input)
}

/// The ``BOOL_AS_INT`` function is used to implement the cast expr in QASM for bool to int.
function BoolAsInt(value : Bool) : Int {
    if value {
        1
    } else {
        0
    }
}

/// The ``BOOL_AS_BIGINT`` function is used to implement the cast expr in QASM for bool to big int.
function BoolAsBigInt(value : Bool) : BigInt {
    if value {
        1L
    } else {
        0L
    }
}

/// The ``BOOL_AS_DOUBLE`` function is used to implement the cast expr in QASM for bool to float.
function BoolAsDouble(value : Bool) : Double {
    if value {
        1.
    } else {
        0.
    }
}

/// The ``RESULT_AS_BOOL`` function is used to implement the cast expr in QASM for bit to bool.
/// This already exists in the Q# library, but is defined as a marker for casts from QASM.
function ResultAsBool(input : Result) : Bool {
    Std.Convert.ResultAsBool(input)
}

/// The ``RESULT_AS_INT`` function is used to implement the cast expr in QASM for bit to int.
function ResultAsInt(input : Result) : Int {
    if Std.Convert.ResultAsBool(input) {
        1
    } else {
        0
    }
}

/// The ``RESULT_AS_BIGINT`` function is used to implement the cast expr in QASM for bit to big int.
function ResultAsBigInt(input : Result) : BigInt {
    if Std.Convert.ResultAsBool(input) {
        1L
    } else {
        0L
    }
}

/// The ``RESULT_AS_DOUBLE`` function is used to implement the cast expr in QASM for result to float.
function ResultAsDouble(value : Result) : Double {
    if value == One {
        1.
    } else {
        0.
    }
}

/// The ``RESULT_ARRAY_AS_BOOL_BE`` function is used to implement the cast expr in QASM for bit[bits] to bool.
/// with big-endian order. This is needed for round-trip conversion for bin ops.
function ResultArrayAsBoolBE(array: Result[]) : Bool {
    for result in array {
        if result == One {
            return true;
        }
    }
    false
}

/// The ``RESULT_ARRAY_AS_RESULT_BE`` function is used to implement the cast expr in QASM for bit[bits] to bit.
/// with big-endian order. This is needed for round-trip conversion for bin ops.
function ResultArrayAsResultBE(array: Result[]) : Result {
    BoolAsResult(ResultArrayAsBoolBE(array))
}

/// The ``RESULT_ARRAY_AS_RESULT_BE`` function is used to implement the cast expr in QASM for bit[bits] to angle[bits].
/// with big-endian order. This is needed for round-trip conversion for bin ops.
function ResultArrayAsAngleBE(array: Result[], bits: Int) : Angle {
    IntAsAngle(ResultArrayAsIntBE(array), bits)
}

/// The ``INT_AS_RESULT_ARRAY_BE`` function is used to implement the cast expr in QASM for int to bit[].
/// with big-endian order. This is needed for round-trip conversion for bin ops.
function IntAsResultArrayBE(number : Int, bits : Int) : Result[] {
    mutable runningValue = number;
    mutable result = [];
    for _ in 1..bits {
        set result += [BoolAsResult((runningValue &&& 1) != 0)];
        set runningValue >>>= 1;
    }
    Std.Arrays.Reversed(result)
}

/// The ``ANGLE_AS_RESULT_ARRAY_BE`` function is used to implement the cast expr in QASM for angle[bits] to bit[bits].
/// with big-endian order. This is needed for round-trip conversion for bin ops.
function AngleAsResultArrayBE(angle : Angle, bits : Int) : Result[] {
    IntAsResultArrayBE(angle.Value, bits)
}

/// The ``BOOL_AS_RESULT_ARRAY_BE`` function is used to implement the cast expr in QASM for bool to bit[bits].
/// with big-endian order. This is needed for round-trip conversion for bin ops.
function BoolAsResultArrayBE(value : Bool, bits: Int) : Result[] {
    IntAsResultArrayBE(BoolAsInt(value), bits)
}

/// The ``RESULT_AS_RESULT_ARRAY_BE`` function is used to implement the cast expr in QASM for bit to bit[bits].
/// with big-endian order. This is needed for round-trip conversion for bin ops.
function ResultAsResultArrayBE(value : Result, bits: Int) : Result[] {
    IntAsResultArrayBE(ResultAsInt(value), bits)
}

/// The ``RESULT_ARRAY_AS_INT_BE`` function is used to implement the cast expr in QASM for bit[] to uint.
/// with big-endian order. This is needed for round-trip conversion for bin ops.
function ResultArrayAsIntBE(results : Result[]) : Int {
    Std.Convert.ResultArrayAsInt(Std.Arrays.Reversed(results))
}

/// The ``DOUBLE_AS_RESULT`` function is used to implement the cast expr in QASM for float to bit.
/// This is needed for round-trip conversion for bin ops.
function DoubleAsResult(value : Double) : Result {
    if Truncate(value) == 0 {
        Zero
    } else {
        One
    }
}

export BoolAsResult, BoolAsInt, BoolAsBigInt, BoolAsDouble, ResultAsBool, ResultAsInt, ResultAsBigInt, IntAsResultArrayBE, ResultArrayAsIntBE,;
