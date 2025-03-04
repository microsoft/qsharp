// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

export GeneratorIndex;
export GeneratorSystem;
export EvolutionGenerator;
export HTermToGenIdx;
export HTermsToGenSys;
export HTermsToGenIdx;
export MultiplexOperationsFromGenerator;

import Std.Math.*;
import Std.Arrays.*;

/// # Summary
/// Represents a single primitive term in the set of all dynamical generators, e.g.
/// Hermitian operators, for which there exists a map from that generator
/// to time-evolution by that generator, through an evolution set function.
///
/// The first element
/// (Int[], Double[]) is indexes that single term -- For instance, the Pauli string
/// XXY with coefficient 0.5 would be indexed by ([1,1,2], [0.5]). Alternatively,
/// Hamiltonians parameterized by a continuous variable, such as X cos φ + Y sin φ,
/// might for instance be represented by ([], [φ]). The second
/// element indexes the subsystem on which the generator acts on.
///
/// # Remarks
/// > [!WARNING]
/// > The interpretation of an `GeneratorIndex` is not defined except
/// > with reference to a particular set of generators.
///
/// # Example
/// Using Pauli evolution set, the operator
/// $\pi X_2 X_5 Y_9$ is represented as:
/// ```qsharp
/// let index = new GeneratorIndex {
///     Term = ([1, 1, 2], [PI()]),
///     Subsystem = [2, 5, 9]
/// };
/// ```
struct GeneratorIndex {
    Term : (Int[], Double[]),
    Subsystem : Int[],
}

/// # Summary
/// Represents a collection of `GeneratorIndex`es.
///
/// We iterate over this
/// collection using a single-index integer, and the size of the
/// collection is assumed to be known.
struct GeneratorSystem {
    NumEntries : Int,
    EntryAt : (Int -> GeneratorIndex)
}

/// # Summary
/// Represents a dynamical generator as a set of simulatable gates and
/// an expansion in terms of that basis.
///
/// Last parameter for number of terms.
struct EvolutionGenerator {
    EvolutionSet : GeneratorIndex -> (Double, Qubit[]) => Unit is Adj + Ctl,
    System : GeneratorSystem
}

/// # Summary
/// Converts a Hamiltonian term to a GeneratorIndex.
///
/// # Input
/// ## term
/// Input data in `(Int[], Double[])` format.
/// ## termType
/// Additional information added to GeneratorIndex.
///
/// # Output
/// A GeneratorIndex representing a Hamiltonian term represented by `term`,
/// together with additional information added by `termType`.
function HTermToGenIdx(term : (Int[], Double[]), termType : Int[]) : GeneratorIndex {
    let (idxFermions, coeff) = term;
    new GeneratorIndex { Term = (termType, coeff), Subsystem = idxFermions }
}


/// # Summary
/// Converts an index to a Hamiltonian term in `(Int[], Double[])[]` data format to a GeneratorIndex.
///
/// # Input
/// ## data
/// Input data in `(Int[], Double[])[]` format.
/// ## termType
/// Additional information added to GeneratorIndex.
/// ## idx
/// Index to a term of the Hamiltonian
///
/// # Output
/// A GeneratorIndex representing a Hamiltonian term represented by `data[idx]`,
/// together with additional information added by `termType`.
function HTermsToGenIdx(data : (Int[], Double[])[], termType : Int[], idx : Int) : GeneratorIndex {
    return HTermToGenIdx(data[idx], termType);
}


/// # Summary
/// Converts a Hamiltonian in `(Int[], Double[])[]` data format to a GeneratorSystem.
///
/// # Input
/// ## data
/// Input data in `(Int[], Double[])[]` format.
/// ## termType
/// Additional information added to GeneratorIndex.
///
/// # Output
/// A GeneratorSystem representing a Hamiltonian represented by the input `data`.
function HTermsToGenSys(data : (Int[], Double[])[], termType : Int[]) : GeneratorSystem {
    new GeneratorSystem { NumEntries = Length(data), EntryAt = HTermsToGenIdx(data, termType, _) }
}


/// # Summary
/// Applies a multiply-controlled unitary operation $U$ that applies a
/// unitary $V_j$ when controlled by n-qubit number state $\ket{j}$.
///
/// $U = \sum^{N-1}_{j=0}\ket{j}\bra{j}\otimes V_j$.
///
/// # Input
/// ## unitaryGenerator
/// A tuple where the first element `Int` is the number of unitaries $N$,
/// and the second element `(Int -> ('T => () is Adj + Ctl))`
/// is a function that takes an integer $j$ in $[0,N-1]$ and outputs the unitary
/// operation $V_j$.
///
/// ## index
/// $n$-qubit control register that encodes number states $\ket{j}$ in
/// little-endian format.
///
/// ## target
/// Generic qubit register that $V_j$ acts on.
///
/// # Remarks
/// `coefficients` will be padded with identity elements if
/// fewer than $2^n$ are specified. This implementation uses
/// $n-1$ auxiliary qubits.
///
/// # References
/// - [ *Andrew M. Childs, Dmitri Maslov, Yunseong Nam, Neil J. Ross, Yuan Su*,
///      arXiv:1711.10980](https://arxiv.org/abs/1711.10980)
operation MultiplexOperationsFromGenerator<'T>(unitaryGenerator : (Int, (Int -> ('T => Unit is Adj + Ctl))), index : Qubit[], target : 'T) : Unit is Ctl + Adj {
    let (nUnitaries, unitaryFunction) = unitaryGenerator;
    let unitaryGeneratorWithOffset = (nUnitaries, 0, unitaryFunction);
    if Length(index) == 0 {
        fail "MultiplexOperations failed. Number of index qubits must be greater than 0.";
    }
    if nUnitaries > 0 {
        let auxiliary = [];
        Adjoint MultiplexOperationsFromGeneratorImpl(unitaryGeneratorWithOffset, auxiliary, index, target);
    }
}

/// # Summary
/// Implementation step of `MultiplexOperationsFromGenerator`.
operation MultiplexOperationsFromGeneratorImpl<'T>(unitaryGenerator : (Int, Int, (Int -> ('T => Unit is Adj + Ctl))), auxiliary : Qubit[], index : Qubit[], target : 'T) : Unit {
    body (...) {
        let nIndex = Length(index);
        let nStates = 2^nIndex;

        let (nUnitaries, unitaryOffset, unitaryFunction) = unitaryGenerator;

        let nUnitariesLeft = MinI(nUnitaries, nStates / 2);
        let nUnitariesRight = MinI(nUnitaries, nStates);

        let leftUnitaries = (nUnitariesLeft, unitaryOffset, unitaryFunction);
        let rightUnitaries = (nUnitariesRight - nUnitariesLeft, unitaryOffset + nUnitariesLeft, unitaryFunction);

        let newControls = Most(index);

        if nUnitaries > 0 {
            if Length(auxiliary) == 1 and nIndex == 0 {
                // Termination case

                (Controlled Adjoint (unitaryFunction(unitaryOffset)))(auxiliary, target);
            } elif Length(auxiliary) == 0 and nIndex >= 1 {
                // Start case
                let newauxiliary = Tail(index);
                if nUnitariesRight > 0 {
                    MultiplexOperationsFromGeneratorImpl(rightUnitaries, [newauxiliary], newControls, target);
                }
                within {
                    X(newauxiliary);
                } apply {
                    MultiplexOperationsFromGeneratorImpl(leftUnitaries, [newauxiliary], newControls, target);
                }
            } else {
                // Recursion that reduces nIndex by 1 and sets Length(auxiliary) to 1.
                let controls = [Tail(index)] + auxiliary;
                use newauxiliary = Qubit();
                use andauxiliary = Qubit[MaxI(0, Length(controls) - 2)];
                within {
                    if Length(controls) == 0 {
                        X(newauxiliary);
                    } elif Length(controls) == 1 {
                        CNOT(Head(controls), newauxiliary);
                    } else {
                        let controls1 = controls[0..0] + andauxiliary;
                        let controls2 = Rest(controls);
                        let targets = andauxiliary + [newauxiliary];
                        for i in IndexRange(controls1) {
                            AND(controls1[i], controls2[i], targets[i]);
                        }
                    }

                } apply {
                    if nUnitariesRight > 0 {
                        MultiplexOperationsFromGeneratorImpl(rightUnitaries, [newauxiliary], newControls, target);
                    }
                    within {
                        (Controlled X)(auxiliary, newauxiliary);
                    } apply {
                        MultiplexOperationsFromGeneratorImpl(leftUnitaries, [newauxiliary], newControls, target);
                    }
                }
            }
        }
    }
    adjoint auto;
    controlled (controlRegister, ...) {
        MultiplexOperationsFromGeneratorImpl(unitaryGenerator, auxiliary + controlRegister, index, target);
    }
    controlled adjoint auto;
}
