import Std.StatePreparation.ApproximatelyPreparePureStateCP;
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export
    PrepareSparseMultiConfigurationalState,
    PrepareUnitaryCoupledClusterState;

import JordanWigner.JordanWignerClusterOperatorEvolutionSet.JordanWignerClusterOperatorEvolutionSet;
import JordanWigner.JordanWignerClusterOperatorEvolutionSet.JordanWignerClusterOperatorGeneratorSystem;
import JordanWigner.Utils.JordanWignerInputState;
import JordanWigner.Utils.TrotterSimulationAlgorithm;
import Std.Arrays.*;
import Std.Convert.ComplexAsComplexPolar;
import Std.Convert.IntAsDouble;
import Std.StatePreparation.PreparePureStateD;
import Std.Math.*;
import Utils.EvolutionGenerator;

operation PrepareTrialState(stateData : (Int, JordanWignerInputState[]), qubits : Qubit[]) : Unit {
    let (stateType, terms) = stateData;

    // State type indexing from FermionHamiltonianStatePrep
    // public enum StateType
    //{
    //    Default = 0, Single_Configurational = 1, Sparse_Multi_Configurational = 2, Unitary_Coupled_Cluster = 3
    //}

    if stateType == 2 { // Sparse_Multi_Configurational
        if IsEmpty(terms) {
            // Do nothing, as there are no terms to prepare.
        } elif Length(terms) == 1 {
            let (_, qubitIndices) = terms[0]!;
            PrepareSingleConfigurationalStateSingleSiteOccupation(qubitIndices, qubits);
        } else {
            PrepareSparseMultiConfigurationalState(qs => I(qs[0]), terms, qubits);
        }
    } elif stateType == 3 { // Unitary_Coupled_Cluster
        let nTerms = Length(terms);
        let trotterStepSize = 1.0;

        // The last term is the reference state.
        let referenceState = PrepareTrialState((2, [terms[nTerms - 1]]), _);

        PrepareUnitaryCoupledClusterState(referenceState, terms[...nTerms - 2], trotterStepSize, qubits);
    } else {
        fail("Unsupported input state.");
    }
}

/// # Summary
/// Simple state preparation of trial state by occupying
/// spin-orbitals
///
/// # Input
/// ## qubitIndices
/// Indices of qubits to be occupied by electrons.
/// ## qubits
/// Qubits of Hamiltonian.
operation PrepareSingleConfigurationalStateSingleSiteOccupation(qubitIndices : Int[], qubits : Qubit[]) : Unit is Adj + Ctl {
    ApplyToEachCA(X, Subarray(qubitIndices, qubits));
}

function PrepareSingleConfigurationalStateSingleSiteOccupationWrapper(qubitIndices : Int[]) : (Qubit[] => Unit is Adj + Ctl) {
    return PrepareSingleConfigurationalStateSingleSiteOccupation(qubitIndices, _);
}

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
    excitations : JordanWignerInputState[],
    qubits : Qubit[]
) : Unit {
    let nExcitations = Length(excitations);

    mutable coefficientsSqrtAbs = [0.0, size = nExcitations];
    mutable coefficientsNewComplexPolar = Repeated(new ComplexPolar { Magnitude = 0.0, Argument = 0.0 }, nExcitations);
    mutable applyFlips = [[], size = nExcitations];

    for idx in 0..nExcitations - 1 {
        let ((r, i), excitation) = excitations[idx]!;
        coefficientsSqrtAbs w/= idx <- Sqrt(AbsComplexPolar(ComplexAsComplexPolar(new Complex { Real = r, Imag = i })));
        coefficientsNewComplexPolar w/= idx <- new ComplexPolar {
            Magnitude = coefficientsSqrtAbs[idx],
            Argument = ArgComplexPolar(ComplexAsComplexPolar(new Complex { Real = r, Imag = i }))
        };
        applyFlips w/= idx <- excitation;
    }

    let nBitsIndices = Ceiling(Lg(IntAsDouble(nExcitations)));

    mutable success = false;
    repeat {
        use auxillary = Qubit[nBitsIndices + 1];
        use flag = Qubit();

        let arr = Mapped(PrepareSingleConfigurationalStateSingleSiteOccupationWrapper, applyFlips);
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
/// Given a set of coefficients and a little-endian encoded quantum register,
/// prepares an state on that register described by the given coefficients.
///
/// # Description
/// This operation prepares an arbitrary quantum
/// state $\ket{\psi}$ with positive coefficients $\alpha_j\ge 0$ from
/// the $n$-qubit computational basis state $\ket{0...0}$.
///
/// The action of U on the all-zeros state is given by
/// $$
/// \begin{align}
///     U \ket{0\cdots 0} = \ket{\psi} = \frac{\sum_{j=0}^{2^n-1}\alpha_j \ket{j}}{\sqrt{\sum_{j=0}^{2^n-1}|\alpha_j|^2}}.
/// \end{align}
/// $$
///
/// # Input
/// ## coefficients
/// Array of up to $2^n$ real coefficients. The $j$th coefficient
/// indexes the number state $\ket{j}$ encoded in little-endian format.
///
/// ## qubits
/// Qubit register encoding number states in little-endian format. This is
/// expected to be initialized in the computational basis state
/// $\ket{0...0}$.
///
/// # Remarks
/// Negative input coefficients $\alpha_j < 0$ will be treated as though
/// positive with value $|\alpha_j|$. `coefficients` will be padded with
/// elements $\alpha_j = 0.0$ if fewer than $2^n$ are specified.
///
/// # Example
/// The following snippet prepares the quantum state $\ket{\psi}=\sqrt{1/8}\ket{0}+\sqrt{7/8}\ket{2}$
/// in the qubit register `qubitsLE`.
/// ```qsharp
/// let amplitudes = [Sqrt(0.125), 0.0, Sqrt(0.875), 0.0];
/// use qubits = Qubit[2];
/// let qubitsLE = LittleEndian(qubits);
/// PrepareArbitraryStateD(amplitudes, qubitsLE);
/// ```
///
/// # References
/// - [Synthesis of Quantum Logic Circuits](https://arxiv.org/abs/quant-ph/0406176)
///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
//Remove
operation PrepareArbitraryStateD_Removed(coefficients : Double[], qubits : Qubit[]) : Unit is Adj + Ctl {
    PreparePureStateD(coefficients, Reversed(qubits));
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
    clusterOperator : JordanWignerInputState[],
    trotterStepSize : Double,
    qubits : Qubit[]
) : Unit {
    let clusterOperatorGeneratorSystem = JordanWignerClusterOperatorGeneratorSystem(clusterOperator);
    let evolutionGenerator = new EvolutionGenerator {
        EvolutionSet = JordanWignerClusterOperatorEvolutionSet(),
        System = clusterOperatorGeneratorSystem
    };
    let trotterOrder = 1;
    let simulationAlgorithm = TrotterSimulationAlgorithm(trotterStepSize, trotterOrder);
    let oracle = simulationAlgorithm(1.0, evolutionGenerator, _);
    initialStatePreparation(qubits);
    oracle(qubits);
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
function MultiplexerBruteForceFromGenerator(unitaryGenerator : (Int, (Int -> (Qubit[] => Unit is Adj + Ctl)))) : ((Qubit[], Qubit[]) => Unit is Adj + Ctl) {
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
operation MultiplexOperationsBruteForceFromGenerator<'T>(unitaryGenerator : (Int, (Int -> ('T => Unit is Adj + Ctl))), index : Qubit[], target : 'T) : Unit is Adj + Ctl {
    let nIndex = Length(index);
    let nStates = 2^nIndex;
    let (nUnitaries, unitaryFunction) = unitaryGenerator;
    for idxOp in 0..MinI(nStates, nUnitaries) - 1 {
        ApplyControlledOnInt(idxOp, unitaryFunction(idxOp), index, target);
    }
}
