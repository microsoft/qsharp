// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Types.FixedPoint;
import Std.Arrays.IsEmpty, Std.Arrays.Rest;
import Std.Diagnostics.Fact, Std.Diagnostics.CheckAllZero;

/// # Summary
/// Asserts that a quantum fixed-point number is
/// initialized to zero.
///
/// # Description
/// This assertion succeeds when all qubits are in state $\ket{0}$,
/// representing that the register encodes the fixed-point number $0.0$.
operation AssertAllZeroFxP(fp : FixedPoint) : Unit {
    Fact(CheckAllZero(fp::Register), "Quantum fixed-point number was not zero.");
}

/// # Summary
/// Assert that all fixed-point numbers in the provided array
/// have identical point positions and qubit numbers.
///
/// # Input
/// ## fixedPoints
/// Array of quantum fixed-point numbers that will be checked for
/// compatibility (using assertions).
function AssertFormatsAreIdenticalFxP(fixedPoints : FixedPoint[]) : Unit {
    if IsEmpty(fixedPoints) {
        return ();
    }
    let (position, register) = fixedPoints[0]!;
    Fact(position > 0, "Point position must be greater than zero.");
    let n = Length(register);
    for fp in Rest(fixedPoints) {
        Fact(fp::IntegerBits == position, "FixedPoint numbers must have identical binary point position.");
        Fact(Length(fp::Register) == n, "FixedPoint numbers must have identical number of qubits.");
    }
}

/// # Summary
/// Assert that all fixed-point numbers in the provided array
/// have identical point positions when counting from the least-
/// significant bit. I.e., number of bits minus point position must
/// be constant for all fixed-point numbers in the array.
///
/// # Input
/// ## fixedPoints
/// Array of quantum fixed-point numbers that will be checked for
/// compatibility (using assertions).
function AssertPointPositionsIdenticalFxP(fixedPoints : FixedPoint[]) : Unit {
    if IsEmpty(fixedPoints) {
        return ();
    }
    let (position, register) = fixedPoints[0]!;
    Fact(position > 0, "Point position must be greater than zero.");
    let n = Length(register);
    for fp in Rest(fixedPoints) {
        Fact((Length(fp::Register) - fp::IntegerBits) == (n - position), "FixedPoint numbers must have identical point alignment.");
    }
}

export AssertAllZeroFxP, AssertFormatsAreIdenticalFxP, AssertPointPositionsIdenticalFxP;