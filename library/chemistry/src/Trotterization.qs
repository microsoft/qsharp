// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export DecomposedIntoTimeStepsCA;
export TrotterStep;
export TrotterSimulationAlgorithm;

import Std.Arrays.*;
import Std.Convert.IntAsDouble;
import Std.Math.*;

import Generators.EvolutionGenerator;

/// # Summary
/// Implementation of the first-order Trotter–Suzuki integrator.
///
/// # Type Parameters
/// ## 'T
/// The type which each time step should act upon; typically, either
/// `Qubit[]` or `Qubit`.
///
/// # Input
/// ## nSteps
/// The number of operations to be decomposed into time steps.
/// ## op
/// An operation which accepts an index input (type `Int`) and a time
/// input (type `Double`) and a quantum register (type `'T`) for decomposition.
/// ## stepSize
/// Multiplier on size of each step of the simulation.
/// ## target
/// A quantum register on which the operations act.
///
/// # Example
/// The following are equivalent:
/// ```qsharp
/// op(0, deltaT, target);
/// op(1, deltaT, target);
/// ```
/// and
/// ```qsharp
/// Trotter1ImplCA((2, op), deltaT, target);
/// ```
operation Trotter1ImplCA<'T>(
    (nSteps : Int, op : ((Int, Double, 'T) => Unit is Adj + Ctl)),
    stepSize : Double,
    target : 'T
) : Unit is Adj + Ctl {

    for idx in 0..nSteps - 1 {
        op(idx, stepSize, target);
    }
}

/// # Summary
/// Implementation of the second-order Trotter–Suzuki integrator.
///
/// # Type Parameters
/// ## 'T
/// The type which each time step should act upon; typically, either
/// `Qubit[]` or `Qubit`.
///
/// # Input
/// ## nSteps
/// The number of operations to be decomposed into time steps.
/// ## op
/// An operation which accepts an index input (type `Int`) and a time
/// input (type `Double`) and a quantum register (type `'T`) for decomposition.
/// ## stepSize
/// Multiplier on size of each step of the simulation.
/// ## target
/// A quantum register on which the operations act.
///
/// # Example
/// The following are equivalent:
/// ```qsharp
/// op(0, deltaT / 2.0, target);
/// op(1, deltaT / 2.0, target);
/// op(1, deltaT / 2.0, target);
/// op(0, deltaT / 2.0, target);
/// ```
/// and
/// ```qsharp
/// Trotter2ImplCA((2, op), deltaT, target);
/// ```
operation Trotter2ImplCA<'T>(
    (nSteps : Int, op : ((Int, Double, 'T) => Unit is Adj + Ctl)),
    stepSize : Double,
    target : 'T
) : Unit is Adj + Ctl {
    for idx in 0..nSteps - 1 {
        op(idx, stepSize * 0.5, target);
    }

    for idx in nSteps - 1.. -1..0 {
        op(idx, stepSize * 0.5, target);
    }
}

/// # Summary
/// Computes Trotter step size in recursive implementation of
/// Trotter simulation algorithm.
///
/// # Remarks
/// This operation uses a different indexing convention than that of
/// [quant-ph/0508139](https://arxiv.org/abs/quant-ph/0508139). In
/// particular, `DecomposedIntoTimeStepsCA(_, 4)` corresponds to the
/// scalar $p_2(\lambda)$ in quant-ph/0508139.
function TrotterStepSize(order : Int) : Double {
    return 1.0 / (4.0 - 4.0^(1.0 / (IntAsDouble(order) - 1.0)));
}


/// # Summary
/// Recursive implementation of even-order Trotter–Suzuki integrator.
///
/// # Type Parameters
/// ## 'T
/// The type which each time step should act upon; typically, either
/// `Qubit[]` or `Qubit`.
///
/// # Input
/// ## order
/// Order of Trotter-Suzuki integrator.
/// ## nSteps
/// The number of operations to be decomposed into time steps.
/// ## op
/// An operation which accepts an index input (type `Int`) and a time
/// input (type `Double`) and a quantum register (type `'T`) for decomposition.
/// ## stepSize
/// Multiplier on size of each step of the simulation.
/// ## target
/// A quantum register on which the operations act.
operation TrotterArbitraryImplCA<'T>(
    order : Int,
    (nSteps : Int, op : ((Int, Double, 'T) => Unit is Adj + Ctl)),
    stepSize : Double,
    target : 'T
) : Unit is Adj + Ctl {
    if (order > 2) {
        let stepSizeOuter = TrotterStepSize(order);
        let stepSizeInner = 1.0 - 4.0 * stepSizeOuter;
        TrotterArbitraryImplCA(order - 2, (nSteps, op), stepSizeOuter * stepSize, target);
        TrotterArbitraryImplCA(order - 2, (nSteps, op), stepSizeOuter * stepSize, target);
        TrotterArbitraryImplCA(order - 2, (nSteps, op), stepSizeInner * stepSize, target);
        TrotterArbitraryImplCA(order - 2, (nSteps, op), stepSizeOuter * stepSize, target);
        TrotterArbitraryImplCA(order - 2, (nSteps, op), stepSizeOuter * stepSize, target);
    } elif (order == 2) {
        Trotter2ImplCA((nSteps, op), stepSize, target);
    } else {
        Trotter1ImplCA((nSteps, op), stepSize, target);
    }
}

/// # Summary
/// Returns an operation implementing the Trotter–Suzuki integrator for
/// a given operation.
///
/// # Type Parameters
/// ## 'T
/// The type which each time step should act upon; typically, either
/// `Qubit[]` or `Qubit`.
///
/// # Input
/// ## nSteps
/// The number of operations to be decomposed into time steps.
/// ## op
/// An operation which accepts an index input (type `Int`) and a time
/// input (type `Double`) for decomposition.
/// ## trotterOrder
/// Selects the order of the Trotter–Suzuki integrator to be used.
/// Order 1 and even orders 2, 4, 6,... are currently supported.
///
/// # Output
/// Returns a unitary implementing the Trotter–Suzuki integrator, where
/// the first parameter `Double` is the integration step size, and the
/// second parameter is the target acted upon.
///
/// # Remarks
/// When called with `order` equal to `1`, this function returns an operation
/// that can be simulated by the lowest-order Trotter–Suzuki integrator
/// $$
/// \begin{align}
///     S_1(\lambda) = \prod_{j = 1}^{m} e^{H_j \lambda},
/// \end{align}
/// $$
/// where we have followed the notation of
/// [quant-ph/0508139](https://arxiv.org/abs/quant-ph/0508139)
/// and let $\lambda$ be the evolution time (represented by the first input
/// of the returned operation), and have let $\{H_j\}_{j = 1}^{m}$ be the
/// set of (skew-Hermitian) dynamical generators being integrated such that
/// `op(j, lambda, _)` is simulated by the unitary operator
/// $e^{H_j \lambda}$.
///
/// Similarly, an `order` of `2` returns the second-order symmetric
/// Trotter–Suzuki integrator
/// $$
/// \begin{align}
///     S_2(\lambda) = \prod_{j = 1}^{m} e^{H_k \lambda / 2}
///                    \prod_{j' = m}^{1} e^{H_{j'} \lambda / 2}.
/// \end{align}
/// $$
///
/// Higher even values of `order` are implemented using the recursive
/// construction of [quant-ph/0508139](https://arxiv.org/abs/quant-ph/0508139).
///
/// # References
/// - [ *D. W. Berry, G. Ahokas, R. Cleve, B. C. Sanders* ](https://arxiv.org/abs/quant-ph/0508139)
function DecomposedIntoTimeStepsCA<'T>(
    (nSteps : Int, op : ((Int, Double, 'T) => Unit is Adj + Ctl)),
    trotterOrder : Int
) : ((Double, 'T) => Unit is Adj + Ctl) {
    if (trotterOrder == 1) {
        return Trotter1ImplCA((nSteps, op), _, _);
    } elif (trotterOrder == 2) {
        return Trotter2ImplCA((nSteps, op), _, _);
    } elif (trotterOrder % 2 == 0) {
        return TrotterArbitraryImplCA(trotterOrder, (nSteps, op), _, _);
    } else {
        fail $"Odd order {trotterOrder} not yet supported.";
    }
}

/// # Summary
/// Implements time-evolution by a term contained in a `GeneratorSystem`.
///
/// # Input
/// ## evolutionGenerator
/// A complete description of the system to be simulated.
/// ## idx
/// Integer index to a term in the described system.
/// ## stepsize
/// Multiplier on duration of time-evolution by term indexed by `idx`.
/// ## qubits
/// Qubits acted on by simulation.
operation TrotterStepImpl(
    evolutionGenerator : EvolutionGenerator,
    idx : Int,
    stepsize : Double,
    qubits : Qubit[]
) : Unit is Adj + Ctl {

    let generatorIndex = evolutionGenerator.System.EntryAt(idx);
    (evolutionGenerator.EvolutionSet(generatorIndex))(stepsize, qubits);
}

/// # Summary
/// Implements a single time-step of time-evolution by the system
/// described in an `EvolutionGenerator` using a Trotter–Suzuki
/// decomposition.
///
/// # Input
/// ## evolutionGenerator
/// A complete description of the system to be simulated.
/// ## trotterOrder
/// Order of Trotter integrator. This must be either 1 or an even number.
/// ## trotterStepSize
/// Duration of simulated time-evolution in single Trotter step.
///
/// # Output
/// Unitary operation that approximates a single step of time-evolution
/// for duration `trotterStepSize`.
function TrotterStep(
    evolutionGenerator : EvolutionGenerator,
    trotterOrder : Int,
    trotterStepSize : Double
) : (Qubit[] => Unit is Adj + Ctl) {

    // The input to DecomposeIntoTimeStepsCA has signature
    // (Int, ((Int, Double, Qubit[]) => () is Adj + Ctl))
    let trotterForm = (
        evolutionGenerator.System.NumEntries,
        TrotterStepImpl(evolutionGenerator, _, _, _)
    );
    return (DecomposedIntoTimeStepsCA(trotterForm, trotterOrder))(trotterStepSize, _);
}

/// # Summary
/// Makes repeated calls to `TrotterStep` to approximate the
/// time-evolution operator exp(_-iHt_).
///
/// # Input
/// ## trotterStepSize
/// Duration of simulated time-evolution in single Trotter step.
/// ## trotterOrder
/// Order of Trotter integrator. This must be either 1 or an even number.
/// ## maxTime
/// Total duration of simulation $t$.
/// ## evolutionGenerator
/// A complete description of the system to be simulated.
/// ## qubits
/// Qubits acted on by simulation.
operation TrotterSimulationAlgorithmImpl(
    trotterStepSize : Double,
    trotterOrder : Int,
    maxTime : Double,
    evolutionGenerator : EvolutionGenerator,
    qubits : Qubit[]
) : Unit is Adj + Ctl {

    let nTimeSlices = Ceiling(maxTime / trotterStepSize);
    let resizedTrotterStepSize = maxTime / IntAsDouble(nTimeSlices);

    for idxTimeSlice in 0..nTimeSlices - 1 {
        (TrotterStep(evolutionGenerator, trotterOrder, resizedTrotterStepSize))(qubits);
    }
}

/// # Summary
/// `SimulationAlgorithm` function that uses a Trotter–Suzuki
/// decomposition to approximate the time-evolution operator _exp(-iHt)_.
///
/// # Input
/// ## trotterStepSize
/// Duration of simulated time-evolution in single Trotter step.
/// ## trotterOrder
/// Order of Trotter integrator. This must be either 1 or an even number.
///
/// # Output
/// A `SimulationAlgorithm` type.
function TrotterSimulationAlgorithm(
    trotterStepSize : Double,
    trotterOrder : Int
) : (Double, EvolutionGenerator, Qubit[]) => Unit is Adj + Ctl {
    return TrotterSimulationAlgorithmImpl(trotterStepSize, trotterOrder, _, _, _);
}
