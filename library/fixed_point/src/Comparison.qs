// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.


import Types.FixedPoint;

/// # Summary
/// Compares two fixed-point numbers stored in quantum registers, and
/// controls a flip on the result.
///
/// # Input
/// ## fp1
/// First fixed-point number to be compared.
/// ## fp2
/// Second fixed-point number to be compared.
/// ## result
/// Result of the comparison. Will be flipped if `fp1 > fp2`.
///
/// # Remarks
/// The current implementation requires the two fixed-point numbers
/// to have the same point position and the same number of qubits.
operation CompareGreaterThanFxP(fp1 : FixedPoint, fp2 : FixedPoint, result : Qubit) : Unit is Adj + Ctl {
    IdenticalFormatFactFxP([fp1, fp2]);
    
    CompareGTSI(SignedLittleEndian(LittleEndian(fp1::Register)), SignedLittleEndian(LittleEndian(fp2::Register)), result);
}

