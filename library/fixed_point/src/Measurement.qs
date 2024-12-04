// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Types.FixedPoint;
import Convert.BoolArrayAsFixedPoint;
import Std.Arrays.ForEach;
import Std.Convert.ResultArrayAsBoolArray;


/// # Summary
/// Measure a fixed-point number, returns its value as Double, and resets
/// all the register to zero.
///
/// # Input
/// ## fp
/// Fixed-point number to measure.
operation MeasureFxP(fp : FixedPoint) : Double {
    let measurements = MResetEachZ(fp::Register);
    let bits = ResultArrayAsBoolArray(measurements);
    return BoolArrayAsFixedPoint(fp::IntegerBits, bits);
}
