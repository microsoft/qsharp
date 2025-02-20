// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

export
    OptimizedBETermIndex,
    OptimizedBEGeneratorSystem,
    OptimizedBlockEncodingGeneratorSystem,
    MixedStatePreparation,
    MixedStatePreparationRequirements,
    PurifiedMixedState,
    PurifiedMixedStateRequirements,
    BlockEncodingByLCU,
    QuantumWalkByQubitization,
    PauliBlockEncoding;

import JordanWigner.OptimizedBEOperator.JordanWignerSelect;
import JordanWigner.OptimizedBEOperator.JordanWignerSelectQubitCount;
import JordanWigner.OptimizedBEOperator.JordanWignerSelectQubitManager;
import JordanWigner.Utils.JWOptimizedHTerms;
import JordanWigner.Utils.MultiplexOperationsFromGenerator;
import JordanWigner.Utils.RangeAsIntArray;
import Std.Arrays.*;
import Std.Math.*;
import Std.Convert.IntAsDouble;
import Std.Arithmetic.ApplyIfGreaterLE;
import Std.StatePreparation.PreparePureStateD;
import Std.Diagnostics.Fact;
import Utils.GeneratorIndex;
import Utils.GeneratorSystem;
import Utils.HTermToGenIdx;
import Utils.IsNotZero;

/// # Summary
/// Term data in the optimized block-encoding algorithm.
struct OptimizedBETermIndex {
    Coefficient : Double,
    UseSignQubit : Bool,
    ZControlRegisterMask : Bool[],
    OptimizedControlRegisterMask : Bool[],
    PauliBases : Int[],
    RegisterIndices : Int[],
}

/// # Summary
/// Function that returns `OptimizedBETermIndex` data for term `n` given an
/// integer `n`, together with the number of terms in the first `Int` and
/// the sum of absolute-values of all term coefficients in the `Double`.
struct OptimizedBEGeneratorSystem {
    NumTerms : Int,
    Norm : Double,
    SelectTerm : (Int -> OptimizedBETermIndex)
}

// Get OptimizedBETermIndex coefficients
function GetOptimizedBETermIndexCoeff(term : OptimizedBETermIndex) : Double {
    let (a, b, c, d, e, f) = term!;
    return a;
}


// Get OptimizedBEGeneratorSystem coefficients
function OptimizedBEGeneratorSystemCoeff(optimizedBEGeneratorSystem : OptimizedBEGeneratorSystem) : Double[] {
    let (nTerms, oneNorm, intToGenIdx) = optimizedBEGeneratorSystem!;
    mutable coefficients = [0.0, size = nTerms];

    for idx in 0..nTerms - 1 {
        coefficients w/= idx <- GetOptimizedBETermIndexCoeff(intToGenIdx(idx));
    }

    return coefficients;
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
function ZTermToPauliMajIdx(term : GeneratorIndex) : OptimizedBETermIndex {
    let ((idxTermType, coeff), idxFermions) = term!;
    let signQubit = coeff[0] < 0.0;
    let selectZControlRegisters = [true];
    let optimizedBEControlRegisters = [];
    let pauliBases = [];
    let indexRegisters = idxFermions;
    return new OptimizedBETermIndex {
        Coefficient = coeff[0],
        UseSignQubit = signQubit,
        ZControlRegisterMask = selectZControlRegisters,
        OptimizedControlRegisterMask = optimizedBEControlRegisters,
        PauliBases = pauliBases,
        RegisterIndices = indexRegisters
    };
}


/// # Summary
/// Converts a GeneratorIndex describing a ZZ term to
/// an expression `GeneratorIndex[]` in terms of Paulis.
///
/// # Input
/// ## term
/// `GeneratorIndex` representing a ZZ term.
///
/// # Output
/// `GeneratorIndex[]` expressing ZZ term as Pauli terms.
function ZZTermToPauliMajIdx(term : GeneratorIndex) : OptimizedBETermIndex {
    let ((idxTermType, coeff), idxFermions) = term!;
    let signQubit = coeff[0] < 0.0;
    let selectZControlRegisters = [true, true];
    let optimizedBEControlRegisters = [];
    let pauliBases = [];
    let indexRegisters = idxFermions;
    return new OptimizedBETermIndex {
        Coefficient = 2.0 * coeff[0],
        UseSignQubit = signQubit,
        ZControlRegisterMask = selectZControlRegisters,
        OptimizedControlRegisterMask = optimizedBEControlRegisters,
        PauliBases = pauliBases,
        RegisterIndices = indexRegisters
    };
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
function PQTermToPauliMajIdx(term : GeneratorIndex) : OptimizedBETermIndex {
    let ((idxTermType, coeff), idxFermions) = term!;
    let sign = coeff[0] < 0.0;
    let selectZControlRegisters = [];
    let optimizedBEControlRegisters = [true, true];
    let pauliBases = [1, 2];
    let indexRegisters = idxFermions;
    return new OptimizedBETermIndex {
        Coefficient = 2.0 * coeff[0],
        UseSignQubit = sign,
        ZControlRegisterMask = selectZControlRegisters,
        OptimizedControlRegisterMask = optimizedBEControlRegisters,
        PauliBases = pauliBases,
        RegisterIndices = indexRegisters
    };
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
function PQandPQQRTermToPauliMajIdx(term : GeneratorIndex) : OptimizedBETermIndex {
    let ((idxTermType, coeff), idxFermions) = term!;
    let sign = coeff[0] < 0.0;

    if Length(idxFermions) == 2 {
        return PQTermToPauliMajIdx(term);
    } else {
        let qubitPidx = idxFermions[0];
        let qubitQidx = idxFermions[1];
        let qubitRidx = idxFermions[3];
        let selectZControlRegisters = [false, true];
        let optimizedBEControlRegisters = [true, false, true];
        let pauliBases = [1, 2];
        let indexRegisters = [qubitPidx, qubitQidx, qubitRidx];
        return new OptimizedBETermIndex {
            Coefficient = 2.0 * coeff[0],
            UseSignQubit = sign,
            ZControlRegisterMask = selectZControlRegisters,
            OptimizedControlRegisterMask = optimizedBEControlRegisters,
            PauliBases = pauliBases,
            RegisterIndices = indexRegisters
        };
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
function V0123TermToPauliMajIdx(term : GeneratorIndex) : OptimizedBETermIndex[] {
    let ((idxTermType, v0123), idxFermions) = term!;
    let qubitsPQ = idxFermions[0..1];
    let qubitsRS = idxFermions[2..3];
    let qubitsPQJW = RangeAsIntArray(qubitsPQ[0] + 1..qubitsPQ[1] - 1);
    let qubitsRSJW = RangeAsIntArray(qubitsRS[0] + 1..qubitsRS[1] - 1);
    let ops = [[1, 1, 1, 1], [1, 1, 2, 2], [1, 2, 1, 2], [1, 2, 2, 1], [2, 2, 2, 2], [2, 2, 1, 1], [2, 1, 2, 1], [2, 1, 1, 2]];
    mutable majIdxes = Repeated(
        new OptimizedBETermIndex {
            Coefficient = 0.0,
            UseSignQubit = false,
            ZControlRegisterMask = [],
            OptimizedControlRegisterMask = [],
            PauliBases = [],
            RegisterIndices = []
        },
        4
    );
    mutable nonZero = 0;
    let selectZControlRegisters = [];
    let optimizedBEControlRegisters = [true, true, true, true];
    let indexRegisters = idxFermions;

    for idxOp in 0..3 {
        if IsNotZero(v0123[idxOp]) {
            let newCoeff = (2.0 * 0.25) * v0123[idxOp];
            majIdxes w/= nonZero <- new OptimizedBETermIndex {
                Coefficient = newCoeff,
                UseSignQubit = v0123[idxOp] < 0.0,
                ZControlRegisterMask = selectZControlRegisters,
                OptimizedControlRegisterMask = optimizedBEControlRegisters,
                PauliBases = ops[idxOp],
                RegisterIndices = indexRegisters
            };
            nonZero = nonZero + 1;
        }
    }

    return majIdxes[0..nonZero - 1];
}


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
function OptimizedBlockEncodingGeneratorSystem(data : JWOptimizedHTerms) : OptimizedBEGeneratorSystem {
    let (ZData, ZZData, PQandPQQRData, h0123Data) = data!;
    mutable majIdxes = Repeated(
        new OptimizedBETermIndex {
            Coefficient = 0.0,
            UseSignQubit = false,
            ZControlRegisterMask = [],
            OptimizedControlRegisterMask = [],
            PauliBases = [],
            RegisterIndices = []
        },
        ((Length(ZData) + Length(ZZData)) + Length(PQandPQQRData)) + 4 * Length(h0123Data)
    );
    mutable startIdx = 0;

    for idx in IndexRange(ZData) {
        // Array of Arrays of Length 1
        majIdxes w/= idx <- ZTermToPauliMajIdx(HTermToGenIdx(ZData[idx], [0]));
    }

    startIdx = Length(ZData);

    for idx in IndexRange(ZZData) {
        // Array of Arrays of Length 1
        majIdxes w/= startIdx + idx <- ZZTermToPauliMajIdx(HTermToGenIdx(ZZData[idx], [1]));
    }

    startIdx = startIdx + Length(ZZData);

    for idx in IndexRange(PQandPQQRData) {

        // Array of Arrays of Length 1
        majIdxes w/= startIdx + idx <- PQandPQQRTermToPauliMajIdx(HTermToGenIdx(PQandPQQRData[idx], [2]));
    }

    startIdx = startIdx + Length(PQandPQQRData);
    mutable finalIdx = startIdx;

    for idx in 0..Length(h0123Data) - 1 {

        // Array of Arrays of Length up to 4
        let genArr = V0123TermToPauliMajIdx(HTermToGenIdx(h0123Data[idx], [3]));

        for idx0123 in IndexRange(genArr) {
            majIdxes w/= finalIdx <- genArr[idx0123];
            finalIdx = finalIdx + 1;
        }
    }

    mutable oneNorm = 0.0;

    for idx in 0..finalIdx - 1 {
        oneNorm = oneNorm + AbsD(GetOptimizedBETermIndexCoeff(majIdxes[idx]));
    }

    let majIdxes = majIdxes[0..finalIdx - 1];
    return new OptimizedBEGeneratorSystem { NumTerms = finalIdx, Norm = oneNorm, SelectTerm = idx -> majIdxes[idx] };
}


operation ToJordanWignerSelectInput(
    idx : Int,
    optimizedBEGeneratorSystem : OptimizedBEGeneratorSystem,
    signQubit : Qubit,
    selectZControlRegisters : Qubit[],
    optimizedBEControlRegisters : Qubit[],
    pauliBasesIdx : Qubit[],
    indexRegisters : Qubit[][]
) : Unit is Adj + Ctl {
    let (nTerms, oneNorm, intToGenIdx) = optimizedBEGeneratorSystem!;
    let (coeff, signQubitSet, selectZControlRegistersSet, OptimizedBEControlRegistersSet, pauliBasesSet, indexRegistersSet) = (intToGenIdx(idx))!;

    // Write bit to apply - signQubit
    if (signQubitSet == true) {
        X(signQubit);
    }

    // Write bit to activate selectZ operator
    for i in IndexRange(selectZControlRegistersSet) {
        if selectZControlRegistersSet[i] {
            X(selectZControlRegisters[i]);
        }
    }

    // Write bit to activate OptimizedBEXY operator
    for i in IndexRange(OptimizedBEControlRegistersSet) {
        if OptimizedBEControlRegistersSet[i] {
            X(optimizedBEControlRegisters[i]);
        }
    }

    // Write bitstring to apply desired XZ... or YZ... Pauli string
    for i in IndexRange(indexRegistersSet) {
        ApplyXorInPlace(indexRegistersSet[i], indexRegisters[i]);
    }

    // Crete state to select uniform superposition of X and Y operators.
    if Length(pauliBasesSet) == 2 {
        // for PQ or PQQR terms, create |00> + |11>
        ApplyXorInPlace(0, pauliBasesIdx);
    } elif Length(pauliBasesSet) == 4 {
        // for PQRS terms, create |abcd> + |a^ b^ c^ d^>
        if pauliBasesSet[2] == 1 and pauliBasesSet[3] == 1 {
            ApplyXorInPlace(1, pauliBasesIdx);
        } elif pauliBasesSet[2] == 2 and pauliBasesSet[3] == 2 {
            ApplyXorInPlace(2, pauliBasesIdx);
        } elif pauliBasesSet[2] == 1 and pauliBasesSet[3] == 2 {
            ApplyXorInPlace(3, pauliBasesIdx);
        } elif pauliBasesSet[2] == 2 and pauliBasesSet[3] == 1 {
            ApplyXorInPlace(4, pauliBasesIdx);
        }
    }
}

operation ToPauliBases(idx : Int, pauliBases : Qubit[]) : Unit is Adj + Ctl {
    let pauliBasesSet = [[1, 1, 1, 1], [1, 1, 2, 2], [1, 2, 1, 2], [1, 2, 2, 1]];
    H(pauliBases[0]);

    if idx > 0 {
        for idxSet in 1..Length(pauliBasesSet[0]) - 1 {
            if (pauliBasesSet[idx - 1])[idxSet] == 2 {
                X(pauliBases[idxSet]);
            }

            CNOT(pauliBases[0], pauliBases[idxSet]);
        }
    }
}

// This prepares the state that selects _JordanWignerSelect_;
operation JordanWignerOptimizedBlockEncodingStatePrep(targetError : Double, optimizedBEGeneratorSystem : OptimizedBEGeneratorSystem, qROMIdxRegister : Qubit[], qROMGarbage : Qubit[], signQubit : Qubit, selectZControlRegisters : Qubit[], optimizedBEControlRegisters : Qubit[], pauliBases : Qubit[], pauliBasesIdx : Qubit[], indexRegisters : Qubit[][]) : Unit is Adj + Ctl {
    let (nTerms, _, _) = optimizedBEGeneratorSystem!;
    let coefficients = OptimizedBEGeneratorSystemCoeff(optimizedBEGeneratorSystem);
    let purifiedState = PurifiedMixedState(targetError, coefficients);
    let unitaryGenerator = (nTerms, idx -> ToJordanWignerSelectInput(idx, optimizedBEGeneratorSystem, _, _, _, _, _));
    let pauliBasesUnitaryGenerator = (5, idx -> (qs => ToPauliBases(idx, qs)));

    purifiedState.Prepare(qROMIdxRegister, [], qROMGarbage);
    MultiplexOperationsFromGenerator(unitaryGenerator, qROMIdxRegister, (signQubit, selectZControlRegisters, optimizedBEControlRegisters, pauliBasesIdx, indexRegisters));
    MultiplexOperationsFromGenerator(pauliBasesUnitaryGenerator, pauliBasesIdx, pauliBases);
}

function JordanWignerOptimizedBlockEncodingQubitManager(targetError : Double, nCoeffs : Int, nZ : Int, nMaj : Int, nIdxRegQubits : Int, ctrlRegister : Qubit[]) : ((Qubit[], Qubit[], Qubit, Qubit[], Qubit[], Qubit[], Qubit[], Qubit[][]), (Qubit, Qubit[], Qubit[], Qubit[], Qubit[][]), Qubit[]) {
    let requirements = PurifiedMixedStateRequirements(targetError, nCoeffs);
    let parts = Partitioned([requirements.NumIndexQubits, requirements.NumGarbageQubits], ctrlRegister);
    let ((qROMIdx, qROMGarbage), rest0) = ((parts[0], parts[1]), parts[2]);
    let ((signQubit, selectZControlRegisters, optimizedBEControlRegisters, pauliBases, indexRegisters, tmp), rest1) = JordanWignerSelectQubitManager(nZ, nMaj, nIdxRegQubits, rest0, []);
    let registers = Partitioned([3], rest1);
    let pauliBasesIdx = registers[0];
    return ((qROMIdx, qROMGarbage, signQubit, selectZControlRegisters, optimizedBEControlRegisters, pauliBases, pauliBasesIdx, indexRegisters), (signQubit, selectZControlRegisters, optimizedBEControlRegisters, pauliBases, indexRegisters), registers[1]);
}

function JordanWignerOptimizedBlockEncodingQubitCount(targetError : Double, nCoeffs : Int, nZ : Int, nMaj : Int, nIdxRegQubits : Int, nTarget : Int) : ((Int, Int), (Int, Int, Int, Int, Int, Int, Int, Int[], Int)) {
    let (nSelectTotal, (a0, a1, a2, a3, a4)) = JordanWignerSelectQubitCount(nZ, nMaj, nIdxRegQubits);
    let (nQROMTotal, b0, b1) = PurifiedMixedStateRequirements(targetError, nCoeffs)!;
    let pauliBasesIdx = 3;
    return (((nSelectTotal + nQROMTotal) + pauliBasesIdx, nTarget), (b0, b1, a0, a1, a2, a3, pauliBasesIdx, a4, nTarget));
}


operation JordanWignerOptimizedBlockEncodingStatePrepWrapper(targetError : Double, nCoeffs : Int, optimizedBEGeneratorSystem : OptimizedBEGeneratorSystem, nZ : Int, nMaj : Int, nIdxRegQubits : Int, ctrlRegister : Qubit[]) : Unit is Adj + Ctl {
    let (statePrepRegister, selectRegister, rest) = JordanWignerOptimizedBlockEncodingQubitManager(targetError, nCoeffs, nZ, nMaj, nIdxRegQubits, ctrlRegister);
    let statePrepOp = JordanWignerOptimizedBlockEncodingStatePrep(targetError, optimizedBEGeneratorSystem, _, _, _, _, _, _, _, _);
    statePrepOp(statePrepRegister);
}


operation JordanWignerOptimizedBlockEncodingSelect(targetError : Double, nCoeffs : Int, optimizedBEGeneratorSystem : OptimizedBEGeneratorSystem, nZ : Int, nMaj : Int, nIdxRegQubits : Int, ctrlRegister : Qubit[], targetRegister : Qubit[]) : Unit is Adj + Ctl {
    let (statePrepRegister, selectRegister, rest) = JordanWignerOptimizedBlockEncodingQubitManager(targetError, nCoeffs, nZ, nMaj, nIdxRegQubits, ctrlRegister);
    let selectOp = JordanWignerSelect(_, _, _, _, _, targetRegister);
    selectOp(selectRegister);
}


function JordanWignerOptimizedBlockEncoding(targetError : Double, data : JWOptimizedHTerms, nSpinOrbitals : Int) : ((Int, Int), (Double, (Qubit[], Qubit[]) => Unit is Adj + Ctl)) {
    let nZ = 2;
    let nMaj = 4;
    let optimizedBEGeneratorSystem = OptimizedBlockEncodingGeneratorSystem(data);
    let (nCoeffs, oneNorm, tmp) = optimizedBEGeneratorSystem!;
    let nIdxRegQubits = Ceiling(Lg(IntAsDouble(nSpinOrbitals)));
    let ((nCtrlRegisterQubits, nTargetRegisterQubits), rest) = JordanWignerOptimizedBlockEncodingQubitCount(targetError, nCoeffs, nZ, nMaj, nIdxRegQubits, nSpinOrbitals);
    let statePrepOp = JordanWignerOptimizedBlockEncodingStatePrepWrapper(targetError, nCoeffs, optimizedBEGeneratorSystem, nZ, nMaj, nIdxRegQubits, _);
    let selectOp = JordanWignerOptimizedBlockEncodingSelect(targetError, nCoeffs, optimizedBEGeneratorSystem, nZ, nMaj, nIdxRegQubits, _, _);
    let blockEncodingReflection = BlockEncodingByLCU(statePrepOp, selectOp);
    return ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, blockEncodingReflection));
}

function JordanWignerOptimizedQuantumWalkByQubitization(targetError : Double, data : JWOptimizedHTerms, nSpinOrbitals : Int) : ((Int, Int), (Double, ((Qubit[], Qubit[]) => Unit is Adj + Ctl))) {
    let ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, blockEncodingReflection)) = JordanWignerOptimizedBlockEncoding(targetError, data, nSpinOrbitals);
    return ((nCtrlRegisterQubits, nTargetRegisterQubits), (oneNorm, QuantumWalkByQubitization(blockEncodingReflection)));
}

/// # Summary
/// Represents a particular mixed state that can be prepared on an index
/// and a garbage register.
///
/// # Input
/// ## Requirements
/// Specifies the size of the qubit registers required to prepare the
/// mixed state represented by this UDT value.
/// ## Norm
/// Specifies the 1-norm of the coefficients used to define this mixed
/// state.
/// ## Prepare
/// An operation that, given an index register, a data register, and a
/// garbage register initially in the $\ket{0}$, $\let{00\dots 0}$, and
/// $\ket{00\dots 0}$ states (respectively),
/// prepares the state represented by this UDT value on those registers.
struct MixedStatePreparation {
    Requirements : MixedStatePreparationRequirements,
    Norm : Double,
    Prepare : ((Qubit[], Qubit[], Qubit[]) => Unit is Adj + Ctl),
}

/// # Summary
/// Represents the number of qubits required in order to prepare a given
/// mixed state.
///
/// # Input
/// ## NTotalQubits
/// The total number of qubits required by the represented state preparation
/// operation.
/// ## NIndexQubits
/// The number of qubits required for the index register used by the
/// represented state preparation operation.
/// ## NGarbageQubits
/// The number of qubits required for the garbage register used by the
/// represented state preparation operation.
struct MixedStatePreparationRequirements {
    NumTotalQubits : Int,
    NumIndexQubits : Int,
    NumGarbageQubits : Int,
}

/// # Summary
/// Returns an operation that prepares a a purification of a given mixed state.
/// A "purified mixed state" refers to states of the form |ψ⟩ = Σᵢ √𝑝ᵢ |𝑖⟩ |garbageᵢ⟩ specified by a vector of
/// coefficients {𝑝ᵢ}. States of this form can be reduced to mixed states ρ ≔ 𝑝ᵢ |𝑖⟩⟨𝑖| by tracing over the "garbage"
/// register (that is, a mixed state that is diagonal in the computational basis).
///
/// See https://arxiv.org/pdf/1805.03662.pdf?page=15 for further discussion.
///
/// # Description
/// Uses the Quantum ROM technique to represent a given density matrix,
/// returning that representation as a state preparation operation.
///
/// In particular, given a list of $N$ coefficients $\alpha_j$, this
/// function returns an operation that uses the Quantum ROM technique to
/// prepare an approximation
/// $$
/// \begin{align}
///     \tilde\rho = \sum_{j = 0}^{N - 1} p_j \ket{j}\bra{j}
/// \end{align}
/// $$
/// of the mixed state
/// $$
/// \begin{align}
///     \rho = \sum_{j = 0}^{N-1} \frac{|\alpha_j|}{\sum_k |\alpha_k|} \ket{j}\bra{j},
/// \end{align}
/// $$
/// where each $p_j$ is an approximation to the given coefficient $\alpha_j$
/// such that
/// $$
/// \begin{align}
///     \left| p_j - \frac{ |\alpha_j| }{ \sum_k |\alpha_k| } \right| \le \frac{\epsilon}{N}
/// \end{align}
/// $$
/// for each $j$.
///
/// When passed an index register and a register of garbage qubits,
/// initially in the state $\ket{0} \ket{00\cdots 0}$, the returned operation
/// prepares both registers into the purification of $\tilde \rho$,
/// $$
/// \begin{align}
///     \sum_{j=0}^{N-1} \sqrt{p_j} \ket{j}\ket{\text{garbage}_j},
/// \end{align}
/// $$
/// such that resetting and deallocating the garbage register enacts the
/// desired preparation to within the target error $\epsilon$.
///
/// # Input
/// ## targetError
/// The target error $\epsilon$.
/// ## coefficients
/// Array of $N$ coefficients specifying the probability of basis states.
/// Negative numbers $-\alpha_j$ will be treated as positive $|\alpha_j|$.
///
/// # Output
/// An operation that prepares $\tilde \rho$ as a purification onto a joint
/// index and garbage register.
///
/// # Remarks
/// The coefficients provided to this operation are normalized following the
/// 1-norm, such that the coefficients are always considered to describe a
/// valid categorical probability distribution.
///
/// # Example
/// The following code snippet prepares an purification of the $3$-qubit state
/// $\rho=\sum_{j=0}^{4}\frac{|\alpha_j|}{\sum_k |\alpha_k|}\ket{j}\bra{j}$, where
/// $\vec\alpha=(1.0, 2.0, 3.0, 4.0, 5.0)$, and the target error is
/// $10^{-3}$:
/// ```qsharp
/// let coefficients = [1.0, 2.0, 3.0, 4.0, 5.0];
/// let targetError = 1e-3;
/// let purifiedState = PurifiedMixedState(targetError, coefficients);
/// using (indexRegister = Qubit[purifiedState.Requirements.NIndexQubits]) {
///     using (garbageRegister = Qubit[purifiedState.Requirements.NGarbageQubits]) {
///         purifiedState.Prepare(LittleEndian(indexRegister), [], garbageRegister);
///     }
/// }
/// ```
///
/// # References
/// - [Encoding Electronic Spectra in Quantum Circuits with Linear T Complexity](https://arxiv.org/abs/1805.03662)
///   Ryan Babbush, Craig Gidney, Dominic W. Berry, Nathan Wiebe, Jarrod McClean, Alexandru Paler, Austin Fowler, Hartmut Neven
function PurifiedMixedState(targetError : Double, coefficients : Double[]) : MixedStatePreparation {
    let nBitsPrecision = -Ceiling(Lg(0.5 * targetError)) + 1;
    let positiveCoefficients = Mapped(AbsD, coefficients);
    let (oneNorm, keepCoeff, altIndex) = QuantumROMDiscretization(nBitsPrecision, positiveCoefficients);
    let nCoeffs = Length(positiveCoefficients);
    let nBitsIndices = Ceiling(Lg(IntAsDouble(nCoeffs)));

    let op = PrepareQuantumROMState(nBitsPrecision, nCoeffs, nBitsIndices, keepCoeff, altIndex, [], _, _, _);
    let qubitCounts = PurifiedMixedStateRequirements(targetError, nCoeffs);
    return new MixedStatePreparation { Requirements = qubitCounts, Norm = oneNorm, Prepare = op };
}

operation PrepareQuantumROMState(
    nBitsPrecision : Int,
    nCoeffs : Int,
    nBitsIndices : Int,
    keepCoeff : Int[],
    altIndex : Int[],
    data : Bool[][],
    indexRegister : Qubit[],
    dataQubits : Qubit[],
    garbageRegister : Qubit[]
) : Unit is Adj + Ctl {
    let garbageIdx0 = nBitsIndices;
    let garbageIdx1 = garbageIdx0 + nBitsPrecision;
    let garbageIdx2 = garbageIdx1 + nBitsPrecision;
    let garbageIdx3 = garbageIdx2 + 1;

    let altIndexRegister = garbageRegister[0..garbageIdx0 - 1];
    let keepCoeffRegister = garbageRegister[garbageIdx0..garbageIdx1 - 1];
    let uniformKeepCoeffRegister = garbageRegister[garbageIdx1..garbageIdx2 - 1];
    let flagQubit = garbageRegister[garbageIdx3 - 1];
    let dataRegister = dataQubits;
    let altDataRegister = garbageRegister[garbageIdx3...];

    // Create uniform superposition over index and alt coeff register.
    PrepareUniformSuperposition(nCoeffs, indexRegister);
    ApplyToEachCA(H, uniformKeepCoeffRegister);

    // Write bitstrings to altIndex and keepCoeff register.
    let unitaryGenerator = (nCoeffs, idx -> WriteQuantumROMBitString(idx, keepCoeff, altIndex, data, _, _, _, _));
    MultiplexOperationsFromGenerator(unitaryGenerator, indexRegister, (keepCoeffRegister, altIndexRegister, dataRegister, altDataRegister));

    // Perform comparison
    ApplyIfGreaterLE(X, uniformKeepCoeffRegister, keepCoeffRegister, flagQubit);

    let indexRegisterSize = Length(indexRegister);

    // Swap in register based on comparison
    let register = indexRegister + dataRegister;
    let altRegister = altIndexRegister + altDataRegister;
    for i in IndexRange(indexRegister) {
        Controlled SWAP([flagQubit], (register[i], altRegister[i]));
    }
}

// Classical processing
// This discretizes the coefficients such that
// |coefficient[i] * oneNorm - discretizedCoefficient[i] * discretizedOneNorm| * nCoeffs <= 2^{1-bitsPrecision}.
function QuantumROMDiscretization(bitsPrecision : Int, coefficients : Double[]) : (Double, Int[], Int[]) {
    let oneNorm = PNorm(1.0, coefficients);
    let nCoefficients = Length(coefficients);
    Fact(bitsPrecision <= 31, $"Bits of precision {bitsPrecision} unsupported. Max is 31.");
    Fact(nCoefficients > 1, "Cannot prepare state with less than 2 coefficients.");
    Fact(oneNorm >= 0.0, "State must have at least one coefficient > 0");

    let barHeight = 2^bitsPrecision - 1;

    mutable altIndex = RangeAsIntArray(0..nCoefficients - 1);
    mutable keepCoeff = Mapped(
        coefficient -> Round((AbsD(coefficient) / oneNorm) * IntAsDouble(nCoefficients) * IntAsDouble(barHeight)),
        coefficients
    );

    // Calculate difference between number of discretized bars vs. maximum
    mutable bars = 0;
    for idxCoeff in IndexRange(keepCoeff) {
        bars += keepCoeff[idxCoeff] - barHeight;
    }

    // Uniformly distribute excess bars across coefficients.
    for idx in 0..AbsI(bars) - 1 {
        keepCoeff w/= idx <- keepCoeff[idx] + (bars > 0 ? -1 | + 1);
    }

    mutable barSink = [];
    mutable barSource = [];

    for idxCoeff in IndexRange(keepCoeff) {
        if keepCoeff[idxCoeff] > barHeight {
            barSource += [idxCoeff];
        } elif keepCoeff[idxCoeff] < barHeight {
            barSink += [idxCoeff];
        }
    }

    for rep in 0..nCoefficients * 10 {
        if Length(barSink) > 0 and Length(barSource) > 0 {
            let idxSink = Tail(barSink);
            let idxSource = Tail(barSource);
            barSink = Most(barSink);
            barSource = Most(barSource);

            keepCoeff w/= idxSource <- keepCoeff[idxSource] - barHeight + keepCoeff[idxSink];
            altIndex w/= idxSink <- idxSource;

            if keepCoeff[idxSource] < barHeight {
                barSink += [idxSource];
            } elif keepCoeff[idxSource] > barHeight {
                barSource += [idxSource];
            }
        } elif Length(barSource) > 0 {
            let idxSource = Tail(barSource);
            barSource = Most(barSource);
            keepCoeff w/= idxSource <- barHeight;
        } else {
            return (oneNorm, keepCoeff, altIndex);
        }
    }

    return (oneNorm, keepCoeff, altIndex);
}

/// # Summary
/// Returns the total number of qubits that must be allocated
/// in order to apply the operation returned by
/// `PurifiedMixedState`.
///
/// # Input
/// ## targetError
/// The target error $\epsilon$.
/// ## nCoefficients
/// The number of coefficients to be specified in preparing a mixed state.
///
/// # Output
/// A description of how many qubits are required in total, and for each of
/// the index and garbage registers used by the
/// `PurifiedMixedState` function.
function PurifiedMixedStateRequirements(targetError : Double, nCoefficients : Int) : MixedStatePreparationRequirements {
    Fact(targetError > 0.0, "targetError must be positive");
    Fact(nCoefficients > 0, "nCoefficients must be positive");

    let nBitsPrecision = -Ceiling(Lg(0.5 * targetError)) + 1;
    let nIndexQubits = Ceiling(Lg(IntAsDouble(nCoefficients)));
    let nGarbageQubits = nIndexQubits + 2 * nBitsPrecision + 1;
    let nTotal = nGarbageQubits + nIndexQubits;
    return new MixedStatePreparationRequirements { NumTotalQubits = nTotal, NumIndexQubits = nIndexQubits, NumGarbageQubits = nGarbageQubits };
}

/// # Summary
/// Encodes an operator of interest into a `BlockEncoding`.
///
/// This constructs a `BlockEncoding` unitary $U=P\cdot V\cdot P^\dagger$ that encodes some
/// operator $H = \sum_{j}|\alpha_j|U_j$ of interest that is a linear combination of
/// unitaries. Typically, $P$ is a state preparation unitary such that
/// $P\ket{0}\_a=\sum_j\sqrt{\alpha_j/\|\vec\alpha\|\_2}\ket{j}\_a$,
/// and $V=\sum_{j}\ket{j}\bra{j}\_a\otimes U_j$.
///
/// # Input
/// ## statePreparation
/// A unitary $P$ that prepares some target state.
/// ## selector
/// A unitary $V$ that encodes the component unitaries of $H$.
///
/// # Output
/// A unitary $U$ acting jointly on registers `a` and `s` that block-
/// encodes $H$, and satisfies $U^\dagger = U$.
///
/// # Remarks
/// This `BlockEncoding` implementation gives it the properties of a
/// `BlockEncodingReflection`.
function BlockEncodingByLCU<'T, 'S>(
    statePreparation : ('T => Unit is Adj + Ctl),
    selector : (('T, 'S) => Unit is Adj + Ctl)
) : (('T, 'S) => Unit is Adj + Ctl) {
    return ApplyBlockEncodingByLCU(statePreparation, selector, _, _);
}

/// # Summary
/// Implementation of `BlockEncodingByLCU`.
operation ApplyBlockEncodingByLCU<'T, 'S>(
    statePreparation : ('T => Unit is Adj + Ctl),
    selector : (('T, 'S) => Unit is Adj + Ctl),
    auxiliary : 'T,
    system : 'S
) : Unit is Adj + Ctl {
    within {
        statePreparation(auxiliary);
    } apply {
        selector(auxiliary, system);
    }
}

/// # Summary
/// Converts a block-encoding reflection into a quantum walk.
///
/// # Description
/// Given a block encoding represented by a unitary $U$
/// that encodes some operator $H$ of interest, converts it into a quantum
/// walk $W$ containing the spectrum of $\pm e^{\pm i\sin^{-1}(H)}$.
///
/// # Input
/// ## blockEncoding
/// A unitary $U$ to be converted into a Quantum
/// walk.
///
/// # Output
/// A quantum walk $W$ acting jointly on registers `a` and `s` that block-
/// encodes $H$, and contains the spectrum of $\pm e^{\pm i\sin^{-1}(H)}$.
///
/// # References
/// - [Hamiltonian Simulation by Qubitization](https://arxiv.org/abs/1610.06546)
///   Guang Hao Low, Isaac L. Chuang
function QuantumWalkByQubitization(blockEncoding : (Qubit[], Qubit[]) => Unit is Adj + Ctl) : ((Qubit[], Qubit[]) => Unit is Adj + Ctl) {
    return ApplyQuantumWalkByQubitization(blockEncoding, _, _);
}

/// # Summary
/// Implementation of `Qubitization`.
operation ApplyQuantumWalkByQubitization(
    blockEncoding : (Qubit[], Qubit[]) => Unit is Adj + Ctl,
    auxiliary : Qubit[],
    system : Qubit[]
) : Unit is Adj + Ctl {
    Exp([PauliI], -0.5 * PI(), [Head(system)]);
    within {
        ApplyToEachCA(X, auxiliary);
    } apply {
        Controlled R1(Rest(auxiliary), (PI(), Head(system)));
    }
    blockEncoding(auxiliary, system);
}

operation WriteQuantumROMBitString(idx : Int, keepCoeff : Int[], altIndex : Int[], data : Bool[][], keepCoeffRegister : Qubit[], altIndexRegister : Qubit[], dataRegister : Qubit[], altDataRegister : Qubit[]) : Unit is Adj + Ctl {
    if keepCoeff[idx] >= 0 {
        ApplyXorInPlace(keepCoeff[idx], keepCoeffRegister);
    }
    ApplyXorInPlace(altIndex[idx], altIndexRegister);
    if Length(dataRegister) > 0 {
        for i in IndexRange(data[idx]) {
            if data[idx][i] {
                X(dataRegister[i]);
            }
        }
        for i in IndexRange(data[altIndex[idx]]) {
            if data[altIndex[idx]][i] {
                X(altDataRegister[i]);
            }
        }
    }
}

/// # Summary
/// Creates a block-encoding unitary for a Hamiltonian.
///
/// The Hamiltonian $H=\sum_{j}\alpha_j P_j$ is described by a
/// sum of Pauli terms $P_j$, each with real coefficient $\alpha_j$.
///
/// # Input
/// ## generatorSystem
/// A `GeneratorSystem` that describes $H$ as a sum of Pauli terms
///
/// # Output
/// ## First parameter
/// The one-norm of coefficients $\alpha=\sum_{j}|\alpha_j|$.
/// ## Second parameter
/// A block encoding unitary $U$ of the Hamiltonian $H$. As this unitary
/// satisfies $U^2 = I$, it is also a reflection.
///
/// # Remarks
/// This is obtained by preparing and unpreparing the state $\sum_{j}\sqrt{\alpha_j/\alpha}\ket{j}$,
/// and constructing a multiply-controlled unitary `PrepareArbitraryStateD` and `MultiplexOperationsFromGenerator`.
function PauliBlockEncoding(generatorSystem : GeneratorSystem) : (Double, (Qubit[], Qubit[]) => Unit is Adj + Ctl) {
    let multiplexer = unitaryGenerator -> MultiplexOperationsFromGenerator(unitaryGenerator, _, _);
    return PauliBlockEncodingInner(generatorSystem, coeff -> (qs => PreparePureStateD(coeff, Reversed(qs))), multiplexer);
}

/// # Summary
/// Creates a block-encoding unitary for a Hamiltonian.
///
/// The Hamiltonian $H=\sum_{j}\alpha_j P_j$ is described by a
/// sum of Pauli terms $P_j$, each with real coefficient $\alpha_j$.
///
/// # Input
/// ## generatorSystem
/// A `GeneratorSystem` that describes $H$ as a sum of Pauli terms
/// ## statePrepUnitary
/// A unitary operation $P$ that prepares $P\ket{0}=\sum_{j}\sqrt{\alpha_j}\ket{j}$ given
/// an array of coefficients $\{\sqrt{\alpha}_j\}$.
/// ## statePrepUnitary
/// A unitary operation $V$ that applies the unitary $V_j$ controlled on index $\ket{j}$,
/// given a function $f: j\rightarrow V_j$.
///
/// # Output
/// ## First parameter
/// The one-norm of coefficients $\alpha=\sum_{j}|\alpha_j|$.
/// ## Second parameter
/// A block encoding unitary $U$ of the Hamiltonian $U$. As this unitary
/// satisfies $U^2 = I$, it is also a reflection.
///
/// # Remarks
/// Example operations the prepare and unpreparing the state $\sum_{j}\sqrt{\alpha_j/\alpha}\ket{j}$,
/// and construct a multiply-controlled unitary are
/// `PrepareArbitraryStateD` and `MultiplexOperationsFromGenerator`.
function PauliBlockEncodingInner(
    generatorSystem : GeneratorSystem,
    statePrepUnitary : (Double[] -> (Qubit[] => Unit is Adj + Ctl)),
    multiplexer : ((Int, (Int -> (Qubit[] => Unit is Adj + Ctl))) -> ((Qubit[], Qubit[]) => Unit is Adj + Ctl))
) : (Double, (Qubit[], Qubit[]) => Unit is Adj + Ctl) {
    let (nTerms, intToGenIdx) = generatorSystem!;
    let op = idx -> Sqrt(AbsD({
        let ((idxPaulis, coeff), idxQubits) = intToGenIdx(idx)!;
        coeff[0]
    }));
    let coefficients = Mapped(op, RangeAsIntArray(0..nTerms-1));
    let oneNorm = PNorm(2.0, coefficients)^2.0;
    let unitaryGenerator = (nTerms, idx -> PauliLCUUnitary(intToGenIdx(idx)));
    let statePreparation = statePrepUnitary(coefficients);
    let selector = multiplexer(unitaryGenerator);
    let blockEncoding = (qs0, qs1) => BlockEncodingByLCU(statePreparation, selector)(qs0, qs1);
    return (oneNorm, blockEncoding);
}

/// # Summary
/// Used in implementation of `PauliBlockEncoding`
function PauliLCUUnitary(generatorIndex : GeneratorIndex) : (Qubit[] => Unit is Adj + Ctl) {
    return ApplyPauliLCUUnitary(generatorIndex, _);
}

/// # Summary
/// Used in implementation of `PauliBlockEncoding`
operation ApplyPauliLCUUnitary(generatorIndex : GeneratorIndex, qubits : Qubit[]) : Unit is Adj + Ctl {
    let ((idxPaulis, coeff), idxQubits) = generatorIndex!;
    let paulis = [PauliI, PauliX, PauliY, PauliZ];
    let pauliString = IntArrayAsPauliArray(idxPaulis);
    let pauliQubits = Subarray(idxQubits, qubits);

    ApplyPauli(pauliString, pauliQubits);

    if (coeff[0] < 0.0) {
        // -1 phase
        Exp([PauliI], PI(), [Head(pauliQubits)]);
    }
}

function IntArrayAsPauliArray(arr : Int[]) : Pauli[] {
    let paulis = [PauliI, PauliX, PauliY, PauliZ];
    mutable pauliString = [];
    for idxP in arr {
        pauliString += [paulis[idxP]];
    }
    pauliString
}
