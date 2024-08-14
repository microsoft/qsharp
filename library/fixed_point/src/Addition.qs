// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.


import Std.Math.*;
import Types.FixedPoint;
import Init.PrepareFxP;
import Facts.IdenticalPointPosFactFxP;
import Signed.*;

/// # Summary
/// Adds a classical constant to a quantum fixed-point number.
///
/// # Input
/// ## constant
/// Constant to add to the quantum fixed-point number.
/// ## fp
/// Fixed-point number to which the constant will
/// be added.
operation AddConstantFxP(constant : Double, fp : FixedPoint) : Unit is Adj + Ctl {
    let n = Length(fp::Register);
    use ys = Qubit[n];
    let tmpFp = FixedPoint(fp::IntegerBits, ys);
    within  { 
        PrepareFxP(constant, tmpFp);
    } apply {
        AddFxP(tmpFp, fp);
    }
}

/// # Summary
/// Adds two fixed-point numbers stored in quantum registers.
///
/// # Description
/// Given two fixed-point registers respectively in states $\ket{f_1}$ and $\ket{f_2}$,
/// performs the operation $\ket{f_1} \ket{f_2} \mapsto \ket{f_1} \ket{f_1 + f_2}$.
///
/// # Input
/// ## fp1
/// First fixed-point number
/// ## fp2
/// Second fixed-point number, will be updated to contain the sum of the
/// two inputs.
///
/// # Remarks
/// The current implementation requires the two fixed-point numbers
/// to have the same point position counting from the least-significant
/// bit, i.e., $n_i$ and $p_i$ must be equal.
operation AddFxP(fp1 : FixedPoint, fp2 : FixedPoint) : Unit is Adj + Ctl {
    IdenticalPointPosFactFxP([fp1, fp2]);

    AddI(fp1::Register, fp2::Register);
}

/// # Summary
/// Computes the additive inverse of `fp`.
///
/// # Input
/// ## fp
/// Fixed-point number to invert.
///
/// # Remarks
/// Numerical inaccuracies may occur depending on the
/// bit-precision of the fixed-point number.
///
/// # See also
/// - Microsoft.Quantum.Arithmetic.SubtractFxP
operation InvertFxP(fp : FixedPoint) : Unit is Adj + Ctl {
    let (_, reg) = fp!;
    Invert2sSI(reg);
}

/// # Summary
/// Computes `minuend - subtrahend` and stores the difference in `minuend`.
///
/// # Input
/// ## subtrahend
/// The subtrahend of the subtraction - the number to be subtracted.
/// ## minuend
/// The minuend of the subtraction - the number from which the other is subtracted.
///
/// # Remarks
/// Computes the difference by inverting `subtrahend` before and after adding
/// it to `minuend`.  Notice that `minuend`, the first argument is updated.
///
/// # See also
/// - Microsoft.Quantum.Arithmetic.AddFxP
/// - Microsoft.Quantum.Arithmetic.InvertFxP
operation SubtractFxP(minuend : FixedPoint, subtrahend : FixedPoint) : Unit is Adj + Ctl {
    within {
        InvertFxP(subtrahend);
    } apply {
        AddFxP(subtrahend, minuend);
    }
}
