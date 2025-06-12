// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export JWBlockEncodingGeneratorSystem;

import Std.Arrays.IndexRange;

import Generators.GeneratorIndex;
import Generators.GeneratorSystem;
import Generators.HTermToGenIdx;
import Utils.IsNotZero;
import Utils.RangeAsIntArray;
import JordanWigner.Data.JWOptimizedHTerms;

// This block encoding for qubitization runs off data optimized for a Jordan–Wigner encoding.
// This collects terms Z, ZZ, PQandPQQR, hpqrs separately.
// This only apples the needed hpqrs XXXX XXYY terms.

// Convention for GeneratorIndex = ((Int[],Double[]), Int[])
// We index single Paulis as 0 for I, 1 for X, 2 for Y, 3 for Z.
// We index Pauli strings with arrays of integers e.g. a = [3,1,1,2] for ZXXY.
// We assume the zeroth element of Double[] is the angle of rotation
// We index the qubits that Pauli strings act on with arrays of integers e.g. q = [2,4,5,8] for Z_2 X_4 X_5, Y_8
// An example of a Pauli string GeneratorIndex is thus ((a,b), q)

// Consider the Hamiltonian H = 0.1 XI + 0.2 IX + 0.3 ZY
// Its GeneratorTerms are (([1],b),[0]), 0.1),  (([1],b),[1]), 0.2),  (([3,2],b),[0,1]), 0.3).

/// # Summary
/// Converts a Hamiltonian described by `JWOptimizedHTerms`
/// to a `GeneratorSystem` expressed in terms of the Pauli
/// `GeneratorIndex`.
///
/// # Input
/// ## data
/// Description of Hamiltonian in `JWOptimizedHTerms` format.
///
/// # Output
/// Representation of Hamiltonian as `GeneratorSystem`.
function JWBlockEncodingGeneratorSystem(data : JWOptimizedHTerms) : GeneratorSystem {
    let ZData = data.HTerm0;
    let ZZData = data.HTerm1;
    let PQandPQQRData = data.HTerm2;
    let h0123Data = data.HTerm3;
    mutable genIdxes = Repeated(
        new GeneratorIndex { Term = ([0], [0.0]), Subsystem = [0] },
        ((Length(ZData) + Length(ZZData)) + 2 * Length(PQandPQQRData)) + 8 * Length(h0123Data)
    );
    mutable startIdx = 0;

    for idx in IndexRange(ZData) {
        // Array of Arrays of Length 1
        genIdxes w/= idx <- (ZTermToPauliGenIdx(HTermToGenIdx(ZData[idx], [0])))[0];
    }

    startIdx = Length(ZData);

    for idx in IndexRange(ZZData) {
        // Array of Arrays of Length 1
        genIdxes w/= startIdx + idx <- (ZZTermToPauliGenIdx(HTermToGenIdx(ZZData[idx], [1])))[0];
    }

    startIdx = startIdx + Length(ZZData);

    for idx in IndexRange(PQandPQQRData) {

        // Array of Arrays of Length 2
        let genArr = PQandPQQRTermToPauliGenIdx(HTermToGenIdx(PQandPQQRData[idx], [2]));
        genIdxes w/= startIdx + 2 * idx <- genArr[0];
        genIdxes w/= (startIdx + 2 * idx) + 1 <- genArr[1];
    }

    startIdx = startIdx + 2 * Length(PQandPQQRData);
    mutable finalIdx = startIdx;

    for idx in 0..Length(h0123Data) - 1 {
        // Array of Arrays of Length up to 8
        let genArr = V0123TermToPauliGenIdx(HTermToGenIdx(h0123Data[idx], [3]));

        for idx0123 in IndexRange(genArr) {
            genIdxes w/= finalIdx <- genArr[idx0123];
            finalIdx += 1;
        }
    }

    let genIdxes = genIdxes[0..finalIdx - 1];
    return new GeneratorSystem { NumEntries = finalIdx, EntryAt = idx -> genIdxes[idx] };
}

/// # Summary
/// Converts a `GeneratorIndex` describing a Z term to
/// an expression `GeneratorIndex[]` in terms of Paulis.
///
/// # Input
/// ## term
/// `GeneratorIndex` representing a Z term.
///
/// # Output
/// `GeneratorIndex[]` expressing Z term as Pauli terms.
function ZTermToPauliGenIdx(term : GeneratorIndex) : GeneratorIndex[] {
    let (_, coeff) = term.Term;
    return [new GeneratorIndex { Term = ([3], coeff), Subsystem = term.Subsystem }];
}

/// # Summary
/// Converts a `GeneratorIndex` describing a ZZ term to
/// an expression `GeneratorIndex[]` in terms of Paulis.
///
/// # Input
/// ## term
/// `GeneratorIndex` representing a ZZ term.
///
/// # Output
/// `GeneratorIndex[]` expressing ZZ term as Pauli terms.
function ZZTermToPauliGenIdx(term : GeneratorIndex) : GeneratorIndex[] {
    let (_, coeff) = term.Term;
    return [new GeneratorIndex { Term = ([3, 3], coeff), Subsystem = term.Subsystem }];
}

/// # Summary
/// Converts a `GeneratorIndex` describing a PQ term to
/// an expression `GeneratorIndex[]` in terms of Paulis
///
/// # Input
/// ## term
/// `GeneratorIndex` representing a PQ term.
///
/// # Output
/// `GeneratorIndex[]` expressing PQ term as Pauli terms.
function PQTermToPauliGenIdx(term : GeneratorIndex) : GeneratorIndex[] {
    let (_, coeff) = term.Term;
    let newCoeff = [coeff[0]];
    let qubitPidx = term.Subsystem[0];
    let qubitQidx = term.Subsystem[1];
    let qubitIndices = RangeAsIntArray(qubitPidx..qubitQidx);
    return [
        new GeneratorIndex { Term = (([1] + Repeated(3, Length(qubitIndices) - 2)) + [1], newCoeff), Subsystem = qubitIndices },
        new GeneratorIndex { Term = (([2] + Repeated(3, Length(qubitIndices) - 2)) + [2], newCoeff), Subsystem = qubitIndices }
    ];
}

/// # Summary
/// Converts a `GeneratorIndex` describing a PQ or PQQR term to
/// an expression `GeneratorIndex[]` in terms of Paulis
///
/// # Input
/// ## term
/// `GeneratorIndex` representing a PQ or PQQR term.
///
/// # Output
/// `GeneratorIndex[]` expressing PQ or PQQR term as Pauli terms.
function PQandPQQRTermToPauliGenIdx(term : GeneratorIndex) : GeneratorIndex[] {
    let (_, coeff) = term.Term;
    let newCoeff = [coeff[0]];

    if Length(term.Subsystem) == 2 {
        return PQTermToPauliGenIdx(term);
    } else {
        let qubitPidx = term.Subsystem[0];
        let qubitQidx = term.Subsystem[1];
        let qubitRidx = term.Subsystem[3];

        if (qubitPidx < qubitQidx and qubitQidx < qubitRidx) {

            // Apply XZ..ZIZ..ZX
            let qubitIndices = RangeAsIntArray(qubitPidx..qubitQidx - 1) + RangeAsIntArray(qubitQidx + 1..qubitRidx);
            return [
                new GeneratorIndex { Term = (([1] + Repeated(3, Length(qubitIndices) - 2)) + [1], newCoeff), Subsystem = qubitIndices },
                new GeneratorIndex { Term = (([2] + Repeated(3, Length(qubitIndices) - 2)) + [2], newCoeff), Subsystem = qubitIndices }
            ];
        } else {

            // Apply ZI..IXZ..ZX or XZ..ZXI..IZ
            let qubitIndices = RangeAsIntArray(qubitPidx..qubitRidx) + [qubitQidx];
            return [
                new GeneratorIndex { Term = (([1] + Repeated(3, Length(qubitIndices) - 3)) + [1, 3], newCoeff), Subsystem = qubitIndices },
                new GeneratorIndex { Term = (([2] + Repeated(3, Length(qubitIndices) - 3)) + [2, 3], newCoeff), Subsystem = qubitIndices }
            ];
        }
    }
}

/// # Summary
/// Converts a `GeneratorIndex` describing a PQRS term to
/// an expression `GeneratorIndex[]` in terms of Paulis
///
/// # Input
/// ## term
/// `GeneratorIndex` representing a PQRS term.
///
/// # Output
/// `GeneratorIndex[]` expressing PQRS term as Pauli terms.
function V0123TermToPauliGenIdx(term : GeneratorIndex) : GeneratorIndex[] {
    let (_, v0123) = term.Term;
    let qubitsPQ = term.Subsystem[0..1];
    let qubitsRS = term.Subsystem[2..3];
    let qubitsPQJW = RangeAsIntArray(qubitsPQ[0] + 1..qubitsPQ[1] - 1);
    let qubitsRSJW = RangeAsIntArray(qubitsRS[0] + 1..qubitsRS[1] - 1);
    let ops = [[1, 1, 1, 1], [1, 1, 2, 2], [1, 2, 1, 2], [2, 1, 1, 2], [2, 2, 2, 2], [2, 2, 1, 1], [2, 1, 2, 1], [1, 2, 2, 1]];
    mutable genIdxes = Repeated(new GeneratorIndex { Term = ([0], [0.0]), Subsystem = [0] }, 8);
    mutable nonZero = 0;

    for idxOp in IndexRange(ops) {
        if (IsNotZero(v0123[idxOp % 4])) {
            let newCoeff = [v0123[idxOp % 4]];
            genIdxes w/= nonZero <- new GeneratorIndex {
                Term = (ops[idxOp] + Repeated(3, Length(qubitsPQJW) + Length(qubitsRSJW)), newCoeff),
                Subsystem = ((qubitsPQ + qubitsRS) + qubitsPQJW) + qubitsRSJW
            };
            nonZero = nonZero + 1;
        }
    }

    return genIdxes[0..nonZero - 1];
}
