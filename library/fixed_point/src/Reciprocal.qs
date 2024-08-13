// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.


open Microsoft.Quantum.Math;
open Microsoft.Quantum.Canon;
open Microsoft.Quantum.Arrays;
open Microsoft.Quantum.Intrinsic;
open Microsoft.Quantum.Diagnostics;
import Types.FixedPoint;
import Signed.Invert2sSI;
import Signed.ComputeReciprocalI;

/// # Summary
/// Computes the reciprocal of a number stored in a quantum register with
/// the fixed-point representation.
///
/// # Description
/// Given a register in the state $\ket{x}$ for a fixed-point number $x$,
/// computes the reciprocal $1 / x$ into the state of the `result`
/// register.
///
/// # Input
/// ## x
/// Fixed-point number to be inverted.
/// ## result
/// Fixed-point number that will hold the result. Must be initialized to $\ket{0.0}$.
operation ComputeReciprocalFxP(x : FixedPoint, result : FixedPoint) : Unit is Adj {
    body (...) {
        Controlled ComputeReciprocalFxP([], (x, result));
    }
    controlled (controls, ...) {
        let (p, xs) = x!;
        let (pRes, rs) = result!;
        let n = Length(xs);

        Fact(CheckAllZero(rs), "Output register must be zero at this point.");
        Fact(p + pRes - 1 + n >= Length(rs), "Output register is too wide.");
        use sign = Qubit();
        use tmpRes = Qubit[2 * n];
        CNOT(Tail(xs), sign);
        (Controlled Invert2sSI)([sign], xs);
        ComputeReciprocalI(xs, tmpRes);
        (Controlled ApplyToEachCA)(controls, (CNOT, Zipped(tmpRes[p + pRes-1 + n-Length(rs)..Min([n + p + pRes, 2 * n-1])], rs)));
        (Controlled Invert2sSI)([sign], ((rs)));
        (Adjoint ComputeReciprocalI)(xs, tmpRes);
        (Controlled Adjoint Invert2sSI)([sign], (xs));
        CNOT(Tail(xs), sign);
    }
}


