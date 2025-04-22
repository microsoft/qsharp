// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import Std.Math.AbsI;

/// The POW function is used to implement the `pow` modifier in QASM for integers.
operation __Pow__<'T>(N: Int, op: ('T => Unit is Adj + Ctl), target : 'T) : Unit {
    ApplyOperationPower(N, () => op(target));
}

/// The ``BARRIER`` function is used to implement the `barrier` statement in QASM.
/// The `@SimulatableIntrinsic` attribute is used to mark the operation for QIR
/// generation.
/// Q# doesn't support barriers, so this is a no-op. We need to figure out what
/// barriers mean in the context of QIR in the future for better support.
@SimulatableIntrinsic()
operation __quantum__qis__barrier__body() : Unit {}


/// The ``BOOL_AS_RESULT`` function is used to implement the cast expr in QASM for bool to bit.
/// This already exists in the Q# library, but is defined as a marker for casts from QASM.
function __BoolAsResult__(input: Bool) : Result {
    Microsoft.Quantum.Convert.BoolAsResult(input)
}

/// The ``BOOL_AS_INT`` function is used to implement the cast expr in QASM for bool to int.
function __BoolAsInt__(value: Bool) : Int {
    if value {
        1
    } else {
        0
    }
}

/// The ``BOOL_AS_BIGINT`` function is used to implement the cast expr in QASM for bool to big int.

function __BoolAsBigInt__(value: Bool) : BigInt {
    if value {
        1L
    } else {
        0L
    }
}

/// The ``BOOL_AS_DOUBLE`` function is used to implement the cast expr in QASM for bool to int.

function __BoolAsDouble__(value: Bool) : Double {
    if value {
        1.
    } else {
        0.
    }
}

/// The ``RESULT_AS_BOOL`` function is used to implement the cast expr in QASM for bit to bool.
/// This already exists in the Q# library, but is defined as a marker for casts from QASM.
function __ResultAsBool__(input: Result) : Bool {
    Microsoft.Quantum.Convert.ResultAsBool(input)
}

/// The ``RESULT_AS_INT`` function is used to implement the cast expr in QASM for bit to bool.
function __ResultAsInt__(input: Result) : Int {
    if Microsoft.Quantum.Convert.ResultAsBool(input) {
        1
    } else {
        0
    }
}

/// The ``RESULT_AS_BIGINT`` function is used to implement the cast expr in QASM for bit to bool.
function __ResultAsBigInt__(input: Result) : BigInt {
    if Microsoft.Quantum.Convert.ResultAsBool(input) {
        1L
    } else {
        0L
    }
}

/// The ``INT_AS_RESULT_ARRAY_BE`` function is used to implement the cast expr in QASM for int to bit[].
/// with big-endian order. This is needed for round-trip conversion for bin ops.
function __IntAsResultArrayBE__(number : Int, bits : Int) : Result[] {
    mutable runningValue = number;
    mutable result = [];
    for _ in 1..bits {
        set result += [__BoolAsResult__((runningValue &&& 1) != 0)];
        set runningValue >>>= 1;
    }
    Microsoft.Quantum.Arrays.Reversed(result)
}

/// The ``RESULT_ARRAY_AS_INT_BE`` function is used to implement the cast expr in QASM for bit[] to uint.
/// with big-endian order. This is needed for round-trip conversion for bin ops.
function __ResultArrayAsIntBE__(results : Result[]) : Int {
     Microsoft.Quantum.Convert.ResultArrayAsInt(Microsoft.Quantum.Arrays.Reversed(results))
}

export __Pow__, __quantum__qis__barrier__body, __BoolAsResult__, __BoolAsInt__, __BoolAsBigInt__, __BoolAsDouble__, __ResultAsBool__, __ResultAsInt__, __ResultAsBigInt__, __IntAsResultArrayBE__, __ResultArrayAsIntBE__, ;
