// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export
    JordanWignerGeneratorSystem,
    JordanWignerFermionEvolutionSet;

import JordanWigner.Utils.JWOptimizedHTerms;
import Std.Arrays.Fold;
import Std.Arrays.IndexRange;
import Std.Arrays.Subarray;
import Utils.GeneratorIndex;
import Utils.GeneratorSystem;
import Utils.HTermsToGenSys;
import Utils.IsNotZero;

// This evolution set runs off data optimized for a Jordan–Wigner encoding.
// This collects terms Z, ZZ, PQandPQQR, hpqrs separately.
// This only apples the needed hpqrs XXXX XXYY terms.
// Operations here are expressed in terms of Exp([...])

// Convention for GeneratorIndex = ((Int[],Double[]), Int[])
// We index single Paulis as 0 for I, 1 for X, 2 for Y, 3 for Z.
// We index Pauli strings with arrays of integers e.g. a = [3,1,1,2] for ZXXY.
// We assume the zeroth element of Double[] is the angle of rotation
// We index the qubits that Pauli strings act on with arrays of integers e.g. q = [2,4,5,8] for Z_2 X_4 X_5, Y_8
// An example of a Pauli string GeneratorIndex is thus ((a,b), q)

// Consider the Hamiltonian H = 0.1 XI + 0.2 IX + 0.3 ZY
// Its GeneratorTerms are (([1],b),[0]), 0.1),  (([1],b),[1]), 0.2),  (([3,2],b),[0,1]), 0.3).

/// # Summary
/// Applies time-evolution by a Z term described by a `GeneratorIndex`.
///
/// # Input
/// ## term
/// `GeneratorIndex` representing a Z term.
/// ## stepSize
/// Duration of time-evolution.
/// ## qubits
/// Qubits of Hamiltonian.
operation ApplyJordanWignerZTerm(term : GeneratorIndex, stepSize : Double, qubits : Qubit[]) : Unit is Adj + Ctl {
    let ((idxTermType, coeff), idxFermions) = term!;
    let angle = (1.0 * coeff[0]) * stepSize;
    let qubit = qubits[idxFermions[0]];
    Exp([PauliZ], angle, [qubit]);
}


/// # Summary
/// Applies time-evolution by a ZZ term described by a `GeneratorIndex`.
///
/// # Input
/// ## term
/// `GeneratorIndex` representing a ZZ term.
/// ## stepSize
/// Duration of time-evolution.
/// ## qubits
/// Qubits of Hamiltonian.
operation ApplyJordanWignerZZTerm(term : GeneratorIndex, stepSize : Double, qubits : Qubit[]) : Unit is Adj + Ctl {
    let ((idxTermType, coeff), idxFermions) = term!;
    let angle = (1.0 * coeff[0]) * stepSize;
    let qubitsZZ = Subarray(idxFermions[0..1], qubits);
    Exp([PauliZ, PauliZ], angle, qubitsZZ);
}


/// # Summary
/// Applies time-evolution by a PQ term described by a `GeneratorIndex`.
///
/// # Input
/// ## term
/// `GeneratorIndex` representing a PQ term.
/// ## stepSize
/// Duration of time-evolution.
/// ## extraParityQubits
/// Optional parity qubits that flip the sign of time-evolution.
/// ## qubits
/// Qubits of Hamiltonian.
operation ApplyJordanWignerPQTerm(term : GeneratorIndex, stepSize : Double, extraParityQubits : Qubit[], qubits : Qubit[]) : Unit is Adj + Ctl {
    let ((idxTermType, coeff), idxFermions) = term!;
    let angle = (1.0 * coeff[0]) * stepSize;
    let qubitsPQ = Subarray(idxFermions[0..1], qubits);
    let qubitsJW = qubits[idxFermions[0] + 1..idxFermions[1] - 1];
    let ops = [[PauliX, PauliX], [PauliY, PauliY]];
    let padding = Repeated(PauliZ, Length(qubitsJW) + Length(extraParityQubits));

    for op in ops {
        Exp(op + padding, angle, qubitsPQ + qubitsJW + extraParityQubits);
    }
}


/// # Summary
/// Applies time-evolution by a PQ or PQQR term described by a `GeneratorIndex`.
///
/// # Input
/// ## term
/// `GeneratorIndex` representing a PQ or PQQR term.
/// ## stepSize
/// Duration of time-evolution.
/// ## qubits
/// Qubits of Hamiltonian.
operation ApplyJordanWignerPQandPQQRTerm(term : GeneratorIndex, stepSize : Double, qubits : Qubit[]) : Unit is Adj + Ctl {
    let ((idxTermType, coeff), idxFermions) = term!;
    let angle = (1.0 * coeff[0]) * stepSize;
    let qubitQidx = idxFermions[1];

    // For all cases, do the same thing:
    // p < r < q (1/4)(1-Z_q)(Z_{r-1,p+1})(X_p X_r + Y_p Y_r) (same as Hermitian conjugate of r < p < q)
    // q < p < r (1/4)(1-Z_q)(Z_{r-1,p+1})(X_p X_r + Y_p Y_r)
    // p < q < r (1/4)(1-Z_q)(Z_{r-1,p+1})(X_p X_r + Y_p Y_r)

    // This amounts to applying a PQ term, followed by same PQ term after a CNOT from q to the parity bit.
    if Length(idxFermions) == 2 {
        let termPR0 = new GeneratorIndex { Term = (idxTermType, [1.0]), Subsystem = idxFermions };
        ApplyJordanWignerPQTerm(termPR0, angle, [], qubits);
    } else {
        if idxFermions[0] < qubitQidx and qubitQidx < idxFermions[3] {
            let termPR1 = new GeneratorIndex { Term = (idxTermType, [1.0]), Subsystem = [idxFermions[0], idxFermions[3] - 1] };
            let excludingQ = if qubitQidx > 0 { qubits[0..qubitQidx-1] + qubits[qubitQidx + 1...] } else { qubits[1...] };
            ApplyJordanWignerPQTerm(termPR1, angle, [], excludingQ);
        } else {
            let termPR1 = new GeneratorIndex { Term = (idxTermType, [1.0]), Subsystem = [0, idxFermions[3] - idxFermions[0]] };
            ApplyJordanWignerPQTerm(termPR1, angle, [qubits[qubitQidx]], qubits[idxFermions[0]..idxFermions[3]]);
        }
    }
}


/// # Summary
/// Applies time-evolution by a PQRS term described by a given index.
///
/// # Input
/// ## term
/// The index representing a PQRS term to be applied.
/// ## stepSize
/// Duration of time-evolution.
/// ## qubits
/// Qubits to apply the given term to.
operation ApplyJordanWigner0123Term(term : GeneratorIndex, stepSize : Double, qubits : Qubit[]) : Unit is Adj + Ctl {
    let ((idxTermType, v0123), idxFermions) = term!;
    let angle = stepSize;
    let qubitsPQ = Subarray(idxFermions[0..1], qubits);
    let qubitsRS = Subarray(idxFermions[2..3], qubits);
    let qubitsPQJW = qubits[idxFermions[0] + 1..idxFermions[1] - 1];
    let qubitsRSJW = qubits[idxFermions[2] + 1..idxFermions[3] - 1];
    let ops = [[PauliX, PauliX, PauliX, PauliX], [PauliX, PauliX, PauliY, PauliY], [PauliX, PauliY, PauliX, PauliY], [PauliY, PauliX, PauliX, PauliY], [PauliY, PauliY, PauliY, PauliY], [PauliY, PauliY, PauliX, PauliX], [PauliY, PauliX, PauliY, PauliX], [PauliX, PauliY, PauliY, PauliX]];

    for idxOp in IndexRange(ops) {
        if (IsNotZero(v0123[idxOp % 4])) {
            Exp(ops[idxOp] + Repeated(PauliZ, Length(qubitsPQJW) + Length(qubitsRSJW)), angle * v0123[idxOp % 4], ((qubitsPQ + qubitsRS) + qubitsPQJW) + qubitsRSJW);
        }
    }
}


/// # Summary
/// Converts a Hamiltonian described by `JWOptimizedHTerms`
/// to a `GeneratorSystem` expressed in terms of the
/// `GeneratorIndex` convention defined in this file.
///
/// # Input
/// ## data
/// Description of Hamiltonian in `JWOptimizedHTerms` format.
///
/// # Output
/// Representation of Hamiltonian as `GeneratorSystem`.
function JordanWignerGeneratorSystem(data : JWOptimizedHTerms) : GeneratorSystem {
    let (ZData, ZZData, PQandPQQRData, h0123Data) = data!;
    let ZGenSys = HTermsToGenSys(ZData, [0]);
    let ZZGenSys = HTermsToGenSys(ZZData, [1]);
    let PQandPQQRGenSys = HTermsToGenSys(PQandPQQRData, [2]);
    let h0123GenSys = HTermsToGenSys(h0123Data, [3]);
    let sum = AddGeneratorSystems(ZGenSys, ZZGenSys);
    let sum = AddGeneratorSystems(sum, PQandPQQRGenSys);
    return AddGeneratorSystems(sum, h0123GenSys);
}

/// # Summary
/// Adds two `GeneratorSystem`s to create a new `GeneratorSystem`.
///
/// # Input
/// ## generatorSystemA
/// The first `GeneratorSystem`.
/// ## generatorSystemB
/// The second `GeneratorSystem`.
///
/// # Output
/// A `GeneratorSystem` representing a system that is the sum of the
/// input generator systems.
function AddGeneratorSystems(generatorSystemA : GeneratorSystem, generatorSystemB : GeneratorSystem) : GeneratorSystem {
    let nTermsA = GetGeneratorSystemNTerms(generatorSystemA);
    let nTermsB = GetGeneratorSystemNTerms(generatorSystemB);
    let generatorIndexFunctionA = GetGeneratorSystemFunction(generatorSystemA);
    let generatorIndexFunctionB = GetGeneratorSystemFunction(generatorSystemB);
    let generatorIndexFunction = idx -> {
        if idx < nTermsA {
            return generatorIndexFunctionA(idx);
        } else {
            return generatorIndexFunctionB(idx - nTermsA);
        }
    };
    return new GeneratorSystem { NumEntries = nTermsA + nTermsB, EntryAt = generatorIndexFunction };
}

/// # Summary
/// Retrieves the number of terms in a `GeneratorSystem`.
///
/// # Input
/// ## generatorSystem
/// The `GeneratorSystem` of interest.
///
/// # Output
/// The number of terms in a `GeneratorSystem`.
function GetGeneratorSystemNTerms(generatorSystem : GeneratorSystem) : Int {
    let (nTerms, generatorIndexFunction) = generatorSystem!;
    return nTerms;
}

/// # Summary
/// Retrieves the `GeneratorIndex` function in a `GeneratorSystem`.
///
/// # Input
/// ## generatorSystem
/// The `GeneratorSystem` of interest.
///
/// # Output
/// An function that indexes each `GeneratorIndex` term in a Hamiltonian.
function GetGeneratorSystemFunction(generatorSystem : GeneratorSystem) : (Int -> GeneratorIndex) {
    let (nTerms, generatorIndexFunction) = generatorSystem!;
    return generatorIndexFunction;
}

/// # Summary
/// Represents a dynamical generator as a set of simulatable gates and an
/// expansion in the JordanWigner basis.
///
/// # Input
/// ## generatorIndex
/// A generator index to be represented as unitary evolution in the JordanWigner.
/// ## stepSize
/// A multiplier on the duration of time-evolution by the term referenced
/// in `generatorIndex`.
/// ## qubits
/// Register acted upon by time-evolution operator.
operation JordanWignerFermionImpl(generatorIndex : GeneratorIndex, stepSize : Double, qubits : Qubit[]) : Unit is Adj + Ctl {
    let ((idxTermType, idxDoubles), idxFermions) = generatorIndex!;
    let termType = idxTermType[0];

    if (termType == 0) {
        ApplyJordanWignerZTerm(generatorIndex, stepSize, qubits);
    } elif (termType == 1) {
        ApplyJordanWignerZZTerm(generatorIndex, stepSize, qubits);
    } elif (termType == 2) {
        ApplyJordanWignerPQandPQQRTerm(generatorIndex, stepSize, qubits);
    } elif (termType == 3) {
        ApplyJordanWigner0123Term(generatorIndex, stepSize, qubits);
    }
}

/// # Summary
/// Represents a dynamical generator as a set of simulatable gates and an
/// expansion in the JordanWigner basis.
///
/// # Output
/// An evolution set function that maps a `GeneratorIndex` for the JordanWigner basis to
/// an evolution unitary operation.
function JordanWignerFermionEvolutionSet() : GeneratorIndex -> (Double, Qubit[]) => Unit is Adj + Ctl {
    generatorIndex -> (stepSize, qubits) => JordanWignerFermionImpl(generatorIndex, stepSize, qubits)
}
