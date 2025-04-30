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

/// The ``BOOL_AS_DOUBLE`` function is used to implement the cast expr in QASM for bool to int.
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

/// The ``RESULT_ARRAY_AS_INT_BE`` function is used to implement the cast expr in QASM for bit[] to uint.
/// with big-endian order. This is needed for round-trip conversion for bin ops.
function ResultArrayAsIntBE(results : Result[]) : Int {
    Std.Convert.ResultArrayAsInt(Std.Arrays.Reversed(results))
}

export BoolAsResult, BoolAsInt, BoolAsBigInt, BoolAsDouble, ResultAsBool, ResultAsInt, ResultAsBigInt, IntAsResultArrayBE, ResultArrayAsIntBE,;
