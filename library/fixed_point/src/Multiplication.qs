// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.


open Microsoft.Quantum.Arrays;
open Microsoft.Quantum.Intrinsic;
open Microsoft.Quantum.Canon;
import Types.FixedPoint;
import Facts.IdenticalFormatFactFxP, Facts.AssertAllZeroFxP;

/// # Summary
/// Multiplies two fixed-point numbers in quantum registers.
///
/// # Input
/// ## fp1
/// First fixed-point number.
/// ## fp2
/// Second fixed-point number.
/// ## result
/// Result fixed-point number, must be in state $\ket{0}$ initially.
///
/// # Remarks
/// The current implementation requires the three fixed-point numbers
/// to have the same point position and the same number of qubits.
operation MultiplyFxP(fp1 : FixedPoint, fp2 : FixedPoint, result : FixedPoint) : Unit is Adj {

    body (...) {
        Controlled MultiplyFxP([], (fp1, fp2, result));
    }
    controlled (controls, ...) {
        IdenticalFormatFactFxP([fp1, fp2, result]);
        AssertAllZeroFxP(result);
        let n = Length(fp1::Register);

        use tmpResult = Qubit[2 * n];
        let xsInt = SignedLittleEndian(LittleEndian(fp1::Register));
        let ysInt = SignedLittleEndian(LittleEndian(fp2::Register));
        let tmpResultInt = SignedLittleEndian(
            LittleEndian(tmpResult)
        );

        within {
            MultiplySI(xsInt, ysInt, tmpResultInt);
        } apply {
            Controlled ApplyToEachCA(controls, (CNOT, Zipped(tmpResult[n - fp1::IntegerBits..2 * n - fp1::IntegerBits - 1], result::Register)));
        }
    }
}

    /// # Summary
/// Squares a fixed-point number.
///
/// # Input
/// ## fp
/// Fixed-point number.
/// ## result
/// Result fixed-point number,
/// must be in state $\ket{0}$ initially.
operation SquareFxP(fp : FixedPoint, result : FixedPoint) : Unit is Adj {
    body (...) {
        Controlled SquareFxP([], (fp, result));
    }
    controlled (controls, ...) {
        IdenticalFormatFactFxP([fp, result]);
        AssertAllZeroFxP(result);
        let n = Length(fp::Register);

        use tmpResult = Qubit[2 * n];
        let xsInt = SignedLittleEndian(LittleEndian(fp::Register));
        let tmpResultInt = SignedLittleEndian(
            LittleEndian(tmpResult)
        );
        within {
            SquareSI(xsInt, tmpResultInt);
        } apply {
            Controlled ApplyToEachCA(controls, (CNOT, Zipped(tmpResult[n - fp::IntegerBits..2 * n - fp::IntegerBits - 1], result::Register)));
        }
    }
}
