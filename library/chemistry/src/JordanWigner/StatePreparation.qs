// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export PrepareSparseMultiConfigurationalState;
export PrepareUnitaryCoupledClusterState;

import Std.Arrays.*;
import Std.Convert.ComplexAsComplexPolar;
import Std.Convert.IntAsDouble;
import Std.Math.*;
import Std.StatePreparation.PreparePureStateD;
import Std.StatePreparation.ApproximatelyPreparePureStateCP;

import JordanWigner.ClusterOperatorEvolutionSet.JWClusterOperatorEvolutionSet;
import JordanWigner.ClusterOperatorEvolutionSet.JWClusterOperatorGeneratorSystem;
import JordanWigner.Data.JWInputState;
import Trotterization.TrotterSimulationAlgorithm;
import Generators.EvolutionGenerator;

/// # Summary
/// Sparse multi-configurational state preparation of trial state by adding excitations
/// to initial trial state.
///
/// # Input
/// ## initialStatePreparation
/// Unitary to prepare initial trial state.
/// ## excitations
/// Excitations of initial trial state represented by
/// the amplitude of the excitation and the qubit indices
/// the excitation acts on.
/// ## qubits
/// Qubits of Hamiltonian.
operation PrepareSparseMultiConfigurationalState(
    initialStatePreparation : (Qubit[] => Unit),
    excitations : JWInputState[],
    qubits : Qubit[]
) : Unit {
    let nExcitations = Length(excitations);

    mutable coefficientsSqrtAbs = [];
    mutable coefficientsNewComplexPolar = [];
    mutable applyFlips = [];

    for idx in 0..nExcitations - 1 {
        let amplitudePolar = ComplexAsComplexPolar(excitations[idx].Amplitude);
        let sqrAbsAmplitude = Sqrt(AbsComplexPolar(amplitudePolar));

        coefficientsSqrtAbs += [sqrAbsAmplitude];
        coefficientsNewComplexPolar += [new ComplexPolar {
            Magnitude = sqrAbsAmplitude,
            Argument = ArgComplexPolar(amplitudePolar)
        }];
        applyFlips += [excitations[idx].FermionIndices];
    }

    let nBitsIndices = Ceiling(Lg(IntAsDouble(nExcitations)));

    mutable success = false;
    repeat {
        use auxillary = Qubit[nBitsIndices + 1];
        use flag = Qubit();

        let arr = Mapped(qubitIndices -> PrepareSingleOccupationsState(qubitIndices, _), applyFlips);
        let multiplexer = MultiplexerBruteForceFromGenerator(nExcitations, idx -> arr[idx]);
        ApproximatelyPreparePureStateCP(0.0, coefficientsNewComplexPolar, Reversed(auxillary));
        multiplexer(auxillary, qubits);
        Adjoint PreparePureStateD(coefficientsSqrtAbs, Reversed(auxillary));
        ApplyControlledOnInt(0, X, auxillary, flag);

        // if measurement outcome one we prepared required state
        let outcome = M(flag);
        success = outcome == One;
        ResetAll(auxillary);
        Reset(flag);
    } until success
    fixup {
        ResetAll(qubits);
    }
}

/// # Summary
/// Unitary coupled-cluster state preparation of trial state
///
/// # Input
/// ## initialStatePreparation
/// Unitary to prepare initial trial state.
/// ## qubits
/// Qubits of Hamiltonian.
operation PrepareUnitaryCoupledClusterState(
    initialStatePreparation : (Qubit[] => Unit),
    clusterOperator : JWInputState[],
    trotterStepSize : Double,
    qubits : Qubit[]
) : Unit {
    let clusterOperatorGeneratorSystem = JWClusterOperatorGeneratorSystem(clusterOperator);
    let evolutionGenerator = new EvolutionGenerator {
        EvolutionSet = JWClusterOperatorEvolutionSet(),
        System = clusterOperatorGeneratorSystem
    };
    let trotterOrder = 1;
    let simulationAlgorithm = TrotterSimulationAlgorithm(trotterStepSize, trotterOrder);
    let oracle = simulationAlgorithm(1.0, evolutionGenerator, _);
    initialStatePreparation(qubits);
    oracle(qubits);
}

operation PrepareTrialState(
    stateData : (Int, JWInputState[]),
    qubits : Qubit[]
) : Unit {
    let (stateType, terms) = stateData;

    // https://github.com/microsoft/QuantumLibraries/blob/main/Chemistry/src/DataModel/TermTypes.cs#L123
    // State type indexing from FermionHamiltonianStatePrep
    // public enum StateType
    //{
    //    Default = 0, Single_Configurational = 1, Sparse_Multi_Configurational = 2, Unitary_Coupled_Cluster = 3
    //}

    if stateType == 2 {
        // Sparse_Multi_Configurational
        if IsEmpty(terms) {
            // Do nothing, as there are no terms to prepare.
        } elif Length(terms) == 1 {
            PrepareSingleOccupationsState(terms[0].FermionIndices, qubits);
        } else {
            PrepareSparseMultiConfigurationalState(qs => I(qs[0]), terms, qubits);
        }
    } elif stateType == 3 {
        // Unitary_Coupled_Cluster
        let nTerms = Length(terms);
        let trotterStepSize = 1.0;

        // The last term is the reference state.
        let referenceState = PrepareTrialState((2, [terms[nTerms - 1]]), _);

        PrepareUnitaryCoupledClusterState(referenceState, terms[...nTerms - 2], trotterStepSize, qubits);
    } else {
        fail ("Unsupported input state.");
    }
}

/// # Summary
/// Simple state preparation of trial state by occupying spin-orbitals
///
/// # Input
/// ## qubitIndices
/// Indices of qubits to be occupied by electrons.
/// ## qubits
/// Qubits of Hamiltonian.
operation PrepareSingleOccupationsState(
    qubitIndices : Int[],
    qubits : Qubit[]
) : Unit is Adj + Ctl {
    ApplyToEachCA(X, Subarray(qubitIndices, qubits));
}

/// # Summary
/// Returns a multiply-controlled unitary operation $U$ that applies a
/// unitary $V_j$ when controlled by n-qubit number state $\ket{j}$.
///
/// $U = \sum^{2^n-1}_{j=0}\ket{j}\bra{j}\otimes V_j$.
///
/// # Input
/// ## unitaryGenerator
/// A tuple where the first element `Int` is the number of unitaries $N$,
/// and the second element `(Int -> ('T => () is Adj + Ctl))`
/// is a function that takes an integer $j$ in $[0,N-1]$ and outputs the unitary
/// operation $V_j$.
///
/// # Output
/// A multiply-controlled unitary operation $U$ that applies unitaries
/// described by `unitaryGenerator`.
function MultiplexerBruteForceFromGenerator(
    unitaryGenerator : (Int, (Int -> (Qubit[] => Unit is Adj + Ctl)))
) : ((Qubit[], Qubit[]) => Unit is Adj + Ctl) {
    return MultiplexOperationsBruteForceFromGenerator(unitaryGenerator, _, _);
}

/// # Summary
/// Applies multiply-controlled unitary operation $U$ that applies a
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
/// fewer than $2^n$ are specified. This version is implemented
/// directly by looping through n-controlled unitary operators.
operation MultiplexOperationsBruteForceFromGenerator<'T>(
    unitaryGenerator : (Int, (Int -> ('T => Unit is Adj + Ctl))),
    index : Qubit[],
    target : 'T
) : Unit is Adj + Ctl {
    let nIndex = Length(index);
    let nStates = 2^nIndex;
    let (nUnitaries, unitaryFunction) = unitaryGenerator;
    for idxOp in 0..MinI(nStates, nUnitaries) - 1 {
        ApplyControlledOnInt(idxOp, unitaryFunction(idxOp), index, target);
    }
}
