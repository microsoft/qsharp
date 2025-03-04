// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

export JWClusterOperatorEvolutionSet;
export JWClusterOperatorGeneratorSystem;

import Std.Arrays.IndexRange;
import Std.Math.Max;
import Std.Math.Min;

import JordanWigner.Data.JordanWignerInputState;
import Generators.GeneratorIndex;
import Generators.GeneratorSystem;

/// # Summary
/// Computes Z component of Jordan–Wigner string between
/// fermion indices in a fermionic operator with an even
/// number of creation / annihilation operators.
///
/// # Input
/// ## nFermions
/// The Number of fermions in the system.
/// ## idxFermions
/// fermionic operator indices.
///
/// # Output
/// Bitstring `Bool[]` that is `true` where a `PauliZ` should be applied.
///
/// # Example
/// ```qsharp
/// let bitString = ComputeJWBitString(5, [0, 3]) ;
/// // bitString is [false, true, true, false, false].
/// ```
function ComputeJWBitString(nFermions : Int, idxFermions : Int[]) : Bool[] {
    if Length(idxFermions) % 2 != 0 {
        fail $"ComputeJordanWignerString failed. `idxFermions` must contain an even number of terms.";
    }

    mutable zString = Repeated(false, nFermions);
    for fermionIdx in idxFermions {
        if fermionIdx >= nFermions {
            fail $"ComputeJordanWignerString failed. fermionIdx {fermionIdx} out of range.";
        }
        // NOTE: This could be optimized
        for idx in 0..fermionIdx {
            zString w/= idx <- not zString[idx];
        }
    }

    for fermionIdx in idxFermions {
        zString w/= fermionIdx <- false;
    }
    return zString;
}

// Identical to `ComputeJWBitString`, except with the map
// false -> PauliI and true -> PauliZ
function ComputeJWPauliZString(nFermions : Int, idxFermions : Int[]) : Pauli[] {
    let bitString = ComputeJWBitString(nFermions, idxFermions);
    mutable pauliString = Repeated(PauliI, Length(bitString));
    for idx in IndexRange(bitString) {
        if bitString[idx] {
            pauliString w/= idx <- PauliZ;
        }
    }
    return pauliString;
}

// Identical to `ComputeJWPauliZString`, except that some
// specified elements are substituted.
function ComputeJWPauliString(
    nFermions : Int,
    idxFermions : Int[],
    pauliReplacements : Pauli[]
) : Pauli[] {

    mutable pauliString = ComputeJWPauliZString(nFermions, idxFermions);

    for idx in IndexRange(idxFermions) {
        let idxFermion = idxFermions[idx];
        let op = pauliReplacements[idx];
        pauliString w/= idxFermion <- op;
    }

    return pauliString;
}

/// # Summary
/// Applies time-evolution by a cluster operator PQ term described by a `GeneratorIndex`.
///
/// # Input
/// ## term
/// `GeneratorIndex` representing a cluster operator PQ term.
/// ## stepSize
/// Duration of time-evolution.
/// ## qubits
/// Qubits of Hamiltonian.
operation ApplyJWClusterOperatorPQTerm(
    term : GeneratorIndex,
    stepSize : Double,
    qubits : Qubit[]
) : Unit is Adj + Ctl {

    let (_, coeff) = term.Term;
    let idxFermions = term.Subsystem;
    let p = idxFermions[0];
    let q = idxFermions[1];
    if p == q {
        fail $"Unitary coupled-cluster PQ failed: indices {p}, {q} must be distinct";
    }
    let angle = 0.5 * coeff[0] * stepSize;
    let ops = [[PauliX, PauliY], [PauliY, PauliX]];
    let signs = [+ 1.0, -1.0];

    for i in IndexRange(ops) {
        let pauliString = ComputeJWPauliString(Length(qubits), idxFermions, ops[i]);
        Exp(pauliString, signs[i] * angle, qubits);
    }
}


/// # Summary
/// Applies time-evolution by a cluster operator PQRS term described by a `GeneratorIndex`.
///
/// # Input
/// ## term
/// `GeneratorIndex` representing a cluster operator PQRS term.
/// ## stepSize
/// Duration of time-evolution.
/// ## qubits
/// Qubits of Hamiltonian.
operation ApplyJWClusterOperatorPQRSTerm(
    term : GeneratorIndex,
    stepSize : Double,
    qubits : Qubit[]
) : Unit is Adj + Ctl {

    let (_, coeff) = term.Term;
    let idxFermions = term.Subsystem;
    let p = idxFermions[0];
    let q = idxFermions[1];
    let r = idxFermions[2];
    let s = idxFermions[3];
    let angle = 0.125 * coeff[0] * stepSize;

    if p == q or p == r or p == s or q == r or q == s or r == s {
        fail ($"Unitary coupled-cluster PQRS failed: indices {p}, {q}, {r}, {s} must be distinct");
    }

    let x = PauliX;
    let y = PauliY;

    let ops = [[y, y, x, y], [x, x, x, y], [x, y, y, y], [y, x, y, y], [x, y, x, x], [y, x, x, x], [y, y, y, x], [x, x, y, x]];
    let (sortedIndices, signs, globalSign) = JWClusterOperatorPQRSTermSigns([p, q, r, s]);

    for i in IndexRange(ops) {
        let pauliString = ComputeJWPauliString(Length(qubits), sortedIndices, ops[i]);
        Exp(pauliString, globalSign * signs[i] * angle, qubits);
    }
}

function JWClusterOperatorPQRSTermSigns(indices : Int[]) : (Int[], Double[], Double) {
    let p = indices[0];
    let q = indices[1];
    let r = indices[2];
    let s = indices[3];
    mutable sorted = [0, 0, 0, 0];
    mutable signs = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    mutable sign = 1.0;

    if (p > q) {
        sign = sign * -1.0;
    }
    if (r > s) {
        sign = sign * -1.0;
    }
    if (Min([p, q]) > Min([r, s])) {
        sign = sign * -1.0;
        sorted = [Min([r, s]), Max([r, s]), Min([p, q]), Max([p, q])];
    } else {
        sorted = [Min([p, q]), Max([p, q]), Min([r, s]), Max([r, s])];
    }
    // sorted is now in the order
    // [p`,q`,r`,s`], where p`<q`; r`<s`, and Min(p`,q`) is smaller than Min(r`,s`).

    let p1 = sorted[0];
    let q1 = sorted[1];
    let r1 = sorted[2];
    let s1 = sorted[3];
    // Case (p,q) < (r,s) and (p,q) > (r,s)
    if (q1 < r1) {
        // p1 < q1 < r1 < s1
        return ([p1, q1, r1, s1], [1.0, -1.0, -1.0, -1.0, 1.0, 1.0, 1.0, -1.0], sign);
    }
    // Case interleaved
    elif (q1 > r1 and q1 < s1) {
        // p1 < r1 < q1 < s1
        return ([p1, r1, q1, s1], [-1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0], sign);
    }
    // Case contained
    elif (q1 > r1 and q1 > s1) {
        // p1 < r1 < s1 < q1
        return ([p1, r1, s1, q1], [1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0, -1.0], sign);
    } else {
        fail ("Completely invalid cluster operator specified.");
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
function JWClusterOperatorGeneratorSystem(
    data : JordanWignerInputState[]
) : GeneratorSystem {
    new GeneratorSystem {
        NumEntries = Length(data),
        EntryAt = JWStateAsGeneratorIndex(data, _)
    }
}

function JWStateAsGeneratorIndex(data : JordanWignerInputState[], idx : Int) : GeneratorIndex {
    let real = data[idx].Amplitude.Real;
    let idxFermions = data[idx].FermionIndices;

    if Length(idxFermions) == 2 {
        // PQ term
        new GeneratorIndex { Term = ([0], [real]), Subsystem = idxFermions }
    } elif Length(idxFermions) == 4 {
        // PQRS term
        new GeneratorIndex { Term = ([2], [real]), Subsystem = idxFermions }
    } else {
        // Any other term in invalid
        new GeneratorIndex { Term = ([-1], [0.0]), Subsystem = [0] }
    }
}

/// # Summary
/// Represents a dynamical generator as a set of simulatable gates and an
/// expansion in the JordanWigner basis.
///
/// # Input
/// ## generatorIndex
/// A generator index to be represented as unitary evolution in the JordanWigner.
/// ## stepSize
/// Dummy variable to match signature of simulation algorithms.
/// ## qubits
/// Register acted upon by time-evolution operator.
operation JWClusterOperatorImpl(
    generatorIndex : GeneratorIndex,
    stepSize : Double,
    qubits : Qubit[]
) : Unit is Adj + Ctl {

    let (idxTermType, _) = generatorIndex.Term;
    let termType = idxTermType[0];

    if termType == 0 {
        ApplyJWClusterOperatorPQTerm(generatorIndex, stepSize, qubits);
    } elif termType == 2 {
        ApplyJWClusterOperatorPQRSTerm(generatorIndex, stepSize, qubits);
    }
}

/// # Summary
/// Represents a dynamical generator as a set of simulatable gates and an
/// expansion in the JordanWigner basis.
///
/// # Output
/// An evolution set function that maps a `GeneratorIndex` for the JordanWigner basis to
/// an evolution unitary operation.
function JWClusterOperatorEvolutionSet() : GeneratorIndex -> (Double, Qubit[]) => Unit is Adj + Ctl {
    generatorIndex -> (stepSize, qubits) => JWClusterOperatorImpl(generatorIndex, stepSize, qubits)
}
