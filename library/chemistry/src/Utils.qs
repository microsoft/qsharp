// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import Std.Math.AbsD;

/// # Summary
/// Checks whether a `Double` number is not approximately zero.
///
/// # Input
/// ## number
/// Number to be checked
///
/// # Output
/// Returns true if `number` has an absolute value greater than `1e-15`.
function IsNotZero(number : Double) : Bool {
    AbsD(number) > 1e-15
}

function RangeAsIntArray(range : Range) : Int[] {
    mutable arr = [];
    for i in range {
        arr += [i];
    }
    return arr;
}
