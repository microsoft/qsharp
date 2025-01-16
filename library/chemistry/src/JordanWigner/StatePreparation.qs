// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export
    PrepareSparseMultiConfigurationalState,
    PrepareArbitraryStateCP,
    ApproximatelyPrepareArbitraryStateCP,
    ApproximatelyMultiplexPauli,
    ApproximatelyApplyDiagonalUnitary,
    ApproximatelyMultiplexZ,
    BlochSphereCoordinates,
    PrepareArbitraryStateD,
    ApproximatelyPrepareArbitraryStateD,
    PrepareUnitaryCoupledClusterState;

import JordanWigner.JordanWignerClusterOperatorEvolutionSet.JordanWignerClusterOperatorEvolutionSet;
import JordanWigner.JordanWignerClusterOperatorEvolutionSet.JordanWignerClusterOperatorGeneratorSystem;
import JordanWigner.Utils.JordanWignerInputState;
import JordanWigner.Utils.TrotterSimulationAlgorithm;
import Std.Arrays.IsEmpty;
import Std.Arrays.Mapped;
import Std.Arrays.Most;
import Std.Arrays.Padded;
import Std.Arrays.Subarray;
import Std.Arrays.Tail;
import Std.Convert.ComplexAsComplexPolar;
import Std.Convert.IntAsDouble;
import Std.Math.*;
import Utils.EvolutionGenerator;

//newtype JordanWignerInputState = ((Double, Double), Int[]);
operation PrepareTrialState(stateData : (Int, JordanWignerInputState[]), qubits : Qubit[]) : Unit {
    let (stateType, terms) = stateData;

    // State type indexing from FermionHamiltonianStatePrep
    // public enum StateType
    //{
    //    Default = 0, Single_Configurational = 1, Sparse_Multi_Configurational = 2, Unitary_Coupled_Cluster = 3
    //}
    if stateType == 2 {
        if IsEmpty(terms) {
            // Do nothing, as there are no terms to prepare.
        } elif Length(terms) == 1 {
            let (_, qubitIndices) = terms[0]!;
            PrepareSingleConfigurationalStateSingleSiteOccupation(qubitIndices, qubits);
        } else {
            PrepareSparseMultiConfigurationalState(qs => I(qs[0]), terms, qubits);
        }
    } elif stateType == 3 {
        let nTerms = Length(terms);
        let trotterStepSize = 1.0;

        // The last term is the reference state.
        let referenceState = PrepareTrialState((2, [terms[nTerms - 1]]), _);

        PrepareUnitaryCoupledClusterState(referenceState, terms[...nTerms - 2], trotterStepSize, qubits);
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
        PrepareArbitraryStateCP(coefficientsNewComplexPolar, auxillary);
        multiplexer(auxillary, qubits);
        Adjoint PrepareArbitraryStateD(coefficientsSqrtAbs, auxillary);
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
/// state $\ket{\psi}$ with complex coefficients $r_j e^{i t_j}$ from
/// the $n$-qubit computational basis state $\ket{0 \cdots 0}$.
/// In particular, the action of this operation can be simulated by the
/// a unitary transformation $U$ which acts on the all-zeros state as
///
/// $$
/// \begin{align}
///     U\ket{0...0}
///         & = \ket{\psi} \\\\
///         & = \frac{
///                 \sum_{j=0}^{2^n-1} r_j e^{i t_j} \ket{j}
///             }{
///                 \sqrt{\sum_{j=0}^{2^n-1} |r_j|^2}
///             }.
/// \end{align}
/// $$
///
/// # Input
/// ## coefficients
/// Array of up to $2^n$ complex coefficients represented by their
/// absolute value and phase $(r_j, t_j)$. The $j$th coefficient
/// indexes the number state $\ket{j}$ encoded in little-endian format.
///
/// ## qubits
/// Qubit register encoding number states in little-endian format. This is
/// expected to be initialized in the computational basis state
/// $\ket{0...0}$.
///
/// # Remarks
/// Negative input coefficients $r_j < 0$ will be treated as though
/// positive with value $|r_j|$. `coefficients` will be padded with
/// elements $(r_j, t_j) = (0.0, 0.0)$ if fewer than $2^n$ are
/// specified.
///
/// # Example
/// The following snippet prepares the quantum state $\ket{\psi}=\sqrt{1/8}\ket{0}+\sqrt{7/8}\ket{2}$
/// in the qubit register `qubitsLE`.
/// ```qsharp
/// use qubits = Qubit();
/// let qubitsLE = LittleEndian([qubits]);
/// PrepareArbitraryStateCP([ComplexPolar(1.0/Sqrt(2.0),0.0),ComplexPolar(1.0/Sqrt(2.0),PI()/2.0)],qubitsLE); // = |i>
/// ```
///
/// # References
/// - [Synthesis of Quantum Logic Circuits](https://arxiv.org/abs/quant-ph/0406176)
///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
operation PrepareArbitraryStateCP(coefficients : ComplexPolar[], qubits : Qubit[]) : Unit is Adj + Ctl {
    ApproximatelyPrepareArbitraryStateCP(0.0, coefficients, qubits);
}

/// # Summary
/// Given a set of coefficients and a little-endian encoded quantum register,
/// prepares an state on that register described by the given coefficients,
/// up to a given approximation tolerance.
///
/// # Description
/// This operation prepares an arbitrary quantum
/// state $\ket{\psi}$ with complex coefficients $r_j e^{i t_j}$ from
/// the $n$-qubit computational basis state $\ket{0 \cdots 0}$.
/// In particular, the action of this operation can be simulated by the
/// a unitary transformation $U$ which acts on the all-zeros state as
///
/// $$
/// \begin{align}
///     U\ket{0...0}
///         & = \ket{\psi} \\\\
///         & = \frac{
///                 \sum_{j=0}^{2^n-1} r_j e^{i t_j} \ket{j}
///             }{
///                 \sqrt{\sum_{j=0}^{2^n-1} |r_j|^2}
///             }.
/// \end{align}
/// $$
///
/// # Input
/// ## tolerance
/// The approximation tolerance to be used when preparing the given state.
///
/// ## coefficients
/// Array of up to $2^n$ complex coefficients represented by their
/// absolute value and phase $(r_j, t_j)$. The $j$th coefficient
/// indexes the number state $\ket{j}$ encoded in little-endian format.
///
/// ## qubits
/// Qubit register encoding number states in little-endian format. This is
/// expected to be initialized in the computational basis state
/// $\ket{0...0}$.
///
/// # Remarks
/// Negative input coefficients $r_j < 0$ will be treated as though
/// positive with value $|r_j|$. `coefficients` will be padded with
/// elements $(r_j, t_j) = (0.0, 0.0)$ if fewer than $2^n$ are
/// specified.
///
/// # References
/// - [Synthesis of Quantum Logic Circuits](https://arxiv.org/abs/quant-ph/0406176)
///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
operation ApproximatelyPrepareArbitraryStateCP(
    tolerance : Double,
    coefficients : ComplexPolar[],
    qubits : Qubit[]
) : Unit is Adj + Ctl {
    (CompileApproximateArbitraryStatePreparation(tolerance, coefficients, Length(qubits)))(qubits);
}

function CompileApproximateArbitraryStatePreparation(
    tolerance : Double,
    coefficients : ComplexPolar[],
    nQubits : Int
) : (Qubit[] => Unit is Adj + Ctl) {
    // pad coefficients at tail length to a power of 2.
    let coefficientsPadded = Padded(-2^nQubits, new ComplexPolar { Magnitude = 0.0, Argument = 0.0 }, coefficients);
    let idxTarget = 0;
    let rngControl =
    // Determine what controls to apply to `op`.
    nQubits > 1 ? (1..(nQubits - 1)) | (1..0);
    let plan = ApproximatelyUnprepareArbitraryStatePlan(
        tolerance,
        coefficientsPadded,
        (rngControl, idxTarget)
    );
    let unprepare = qs => for op in plan {
        op(qs);
    };
    return Adjoint unprepare;
}

/// # Summary
/// Implementation step of arbitrary state preparation procedure.
function ApproximatelyUnprepareArbitraryStatePlan(
    tolerance : Double,
    coefficients : ComplexPolar[],
    (rngControl : Range, idxTarget : Int)
) : (Qubit[] => Unit is Adj + Ctl)[] {
    mutable plan = [];

    // For each 2D block, compute disentangling single-qubit rotation parameters
    let (disentanglingY, disentanglingZ, newCoefficients) = StatePreparationSBMComputeCoefficients(coefficients);
    if (AnyOutsideToleranceD(tolerance, disentanglingZ)) {
        plan += [ApplyMultiplexStep(tolerance, disentanglingZ, PauliZ, (rngControl, idxTarget), _)];
    }
    if (AnyOutsideToleranceD(tolerance, disentanglingY)) {
        plan += [ApplyMultiplexStep(tolerance, disentanglingY, PauliY, (rngControl, idxTarget), _)];
    }

    // target is now in |0> state up to the phase given by arg of newCoefficients.

    // Continue recursion while there are control qubits.
    if (IsRangeEmpty(rngControl)) {
        let (abs, arg) = newCoefficients[0]!;
        if (AbsD(arg) > tolerance) {
            plan += [ApplyGlobalRotationStep(-1.0 * arg, idxTarget, _)];
        }
    } elif (AnyOutsideToleranceCP(tolerance, newCoefficients)) {
        let newControl = (RangeStart(rngControl) + 1)..RangeStep(rngControl)..RangeEnd(rngControl);
        let newTarget = RangeStart(rngControl);
        plan += ApproximatelyUnprepareArbitraryStatePlan(tolerance, newCoefficients, (newControl, newTarget));
    }

    return plan;
}

operation ApplyMultiplexStep(
    tolerance : Double,
    disentangling : Double[],
    axis : Pauli,
    (rngControl : Range, idxTarget : Int),
    register : Qubit[]
) : Unit is Adj + Ctl {
    let actualControl = register[rngControl];
    ApproximatelyMultiplexPauli(tolerance, disentangling, axis, actualControl, register[idxTarget]);
}

/// # Summary
/// Applies a Pauli rotation conditioned on an array of qubits, truncating
/// small rotation angles according to a given tolerance.
///
/// # Description
/// This applies a multiply controlled unitary operation that performs
/// rotations by angle $\theta_j$ about single-qubit Pauli operator $P$
/// when controlled by the $n$-qubit number state $\ket{j}$.
/// In particular, the action of this operation is represented by the
/// unitary
///
/// $$
/// \begin{align}
///     U = \sum^{2^n - 1}_{j=0} \ket{j}\bra{j} \otimes e^{i P \theta_j}.
/// \end{align}
/// ##
///
/// # Input
/// ## tolerance
/// A tolerance below which small coefficients are truncated.
///
/// ## coefficients
/// Array of up to $2^n$ coefficients $\theta_j$. The $j$th coefficient
/// indexes the number state $\ket{j}$ encoded in little-endian format.
///
/// ## pauli
/// Pauli operator $P$ that determines axis of rotation.
///
/// ## control
/// $n$-qubit control register that encodes number states $\ket{j}$ in
/// little-endian format.
///
/// ## target
/// Single qubit register that is rotated by $e^{i P \theta_j}$.
///
/// # Remarks
/// `coefficients` will be padded with elements $\theta_j = 0.0$ if
/// fewer than $2^n$ are specified.
operation ApproximatelyMultiplexPauli(tolerance : Double, coefficients : Double[], pauli : Pauli, control : Qubit[], target : Qubit) : Unit is Adj + Ctl {
    if pauli == PauliZ {
        let op = ApproximatelyMultiplexZ(tolerance, coefficients, control, _);
        op(target);
    } elif pauli == PauliX {
        let op = ApproximatelyMultiplexPauli(tolerance, coefficients, PauliZ, control, _);
        within {
            H(target);
        } apply {
            op(target);
        }
    } elif pauli == PauliY {
        let op = ApproximatelyMultiplexPauli(tolerance, coefficients, PauliX, control, _);
        within {
            Adjoint S(target);
        } apply {
            op(target);
        }
    } elif pauli == PauliI {
        ApproximatelyApplyDiagonalUnitary(tolerance, coefficients, control);
    } else {
        fail $"MultiplexPauli failed. Invalid pauli {pauli}.";
    }
}

/// # Summary
/// Applies an array of complex phases to numeric basis states of a register
/// of qubits, truncating small rotation angles according to a given
/// tolerance.
///
/// # Description
/// This operation implements a diagonal unitary that applies a complex phase
/// $e^{i \theta_j}$ on the $n$-qubit number state $\ket{j}$.
/// In particular, this operation can be represented by the unitary
///
/// $$
/// \begin{align}
///     U = \sum^{2^n-1}_{j=0}e^{i\theta_j}\ket{j}\bra{j}.
/// \end{align}
/// $$
///
/// # Input
/// ## tolerance
/// A tolerance below which small coefficients are truncated.
///
/// ## coefficients
/// Array of up to $2^n$ coefficients $\theta_j$. The $j$th coefficient
/// indexes the number state $\ket{j}$ encoded in little-endian format.
///
/// ## qubits
/// $n$-qubit control register that encodes number states $\ket{j}$ in
/// little-endian format.
///
/// # Remarks
/// `coefficients` will be padded with elements $\theta_j = 0.0$ if
/// fewer than $2^n$ are specified.
///
/// # References
/// - [Synthesis of Quantum Logic Circuits](https://arxiv.org/abs/quant-ph/0406176)
///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
operation ApproximatelyApplyDiagonalUnitary(tolerance : Double, coefficients : Double[], qubits : Qubit[]) : Unit is Adj + Ctl {
    if IsEmpty(qubits) {
        fail "operation ApplyDiagonalUnitary -- Number of qubits must be greater than 0.";
    }

    // pad coefficients length at tail to a power of 2.
    let coefficientsPadded = Padded(-2^Length(qubits), 0.0, coefficients);

    // Compute new coefficients.
    let (coefficients0, coefficients1) = MultiplexZCoefficients(coefficientsPadded);
    ApproximatelyMultiplexZ(tolerance, coefficients1, Most(qubits), Tail(qubits));

    if Length(coefficientsPadded) == 2 {
        // Termination case
        if AbsD(coefficients0[0]) > tolerance {
            Exp([PauliI], 1.0 * coefficients0[0], qubits);
        }
    } else {
        ApproximatelyApplyDiagonalUnitary(tolerance, coefficients0, Most(qubits));
    }
}

/// # Summary
/// Applies a Pauli Z rotation conditioned on an array of qubits, truncating
/// small rotation angles according to a given tolerance.
///
/// # Description
/// This applies the multiply controlled unitary operation that performs
/// rotations by angle $\theta_j$ about single-qubit Pauli operator $Z$
/// when controlled by the $n$-qubit number state $\ket{j}$.
/// In particular, this operation can be represented by the unitary
///
/// $$
/// \begin{align}
///     U = \sum^{2^n-1}_{j=0} \ket{j}\bra{j} \otimes e^{i Z \theta_j}.
/// \end{align}
/// $$
///
/// # Input
/// ## tolerance
/// A tolerance below which small coefficients are truncated.
///
/// ## coefficients
/// Array of up to $2^n$ coefficients $\theta_j$. The $j$th coefficient
/// indexes the number state $\ket{j}$ encoded in little-endian format.
///
/// ## control
/// $n$-qubit control register that encodes number states $\ket{j}$ in
/// little-endian format.
///
/// ## target
/// Single qubit register that is rotated by $e^{i P \theta_j}$.
///
/// # Remarks
/// `coefficients` will be padded with elements $\theta_j = 0.0$ if
/// fewer than $2^n$ are specified.
///
/// # References
/// - [Synthesis of Quantum Logic Circuits](https://arxiv.org/abs/quant-ph/0406176)
///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
operation ApproximatelyMultiplexZ(tolerance : Double, coefficients : Double[], control : Qubit[], target : Qubit) : Unit is Adj + Ctl {
    body (...) {
        // pad coefficients length at tail to a power of 2.
        let coefficientsPadded = Padded(-2^Length(control), 0.0, coefficients);

        if Length(coefficientsPadded) == 1 {
            // Termination case
            if AbsD(coefficientsPadded[0]) > tolerance {
                Exp([PauliZ], coefficientsPadded[0], [target]);
            }
        } else {
            // Compute new coefficients.
            let (coefficients0, coefficients1) = MultiplexZCoefficients(coefficientsPadded);
            ApproximatelyMultiplexZ(tolerance, coefficients0, Most(control), target);
            if AnyOutsideToleranceD(tolerance, coefficients1) {
                within {
                    CNOT(Tail(control), target);
                } apply {
                    ApproximatelyMultiplexZ(tolerance, coefficients1, Most(control), target);
                }
            }
        }
    }

    controlled (controlRegister, ...) {
        // pad coefficients length to a power of 2.
        let coefficientsPadded = Padded(2^(Length(control) + 1), 0.0, Padded(-2^Length(control), 0.0, coefficients));
        let (coefficients0, coefficients1) = MultiplexZCoefficients(coefficientsPadded);
        ApproximatelyMultiplexZ(tolerance, coefficients0, control, target);
        if AnyOutsideToleranceD(tolerance, coefficients1) {
            within {
                Controlled X(controlRegister, target);
            } apply {
                ApproximatelyMultiplexZ(tolerance, coefficients1, control, target);
            }
        }
    }
}

/// # Summary
/// Implementation step of multiply-controlled Z rotations.
function MultiplexZCoefficients(coefficients : Double[]) : (Double[], Double[]) {
    let newCoefficientsLength = Length(coefficients) / 2;
    mutable coefficients0 = [0.0, size = newCoefficientsLength];
    mutable coefficients1 = [0.0, size = newCoefficientsLength];

    for idxCoeff in 0..newCoefficientsLength - 1 {
        coefficients0 w/= idxCoeff <- 0.5 * (coefficients[idxCoeff] + coefficients[idxCoeff + newCoefficientsLength]);
        coefficients1 w/= idxCoeff <- 0.5 * (coefficients[idxCoeff] - coefficients[idxCoeff + newCoefficientsLength]);
    }

    return (coefficients0, coefficients1);
}

operation ApplyGlobalRotationStep(
    angle : Double,
    idxTarget : Int,
    register : Qubit[]
) : Unit is Adj + Ctl {
    Exp([PauliI], angle, [register[idxTarget]]);
}

function AnyOutsideToleranceD(tolerance : Double, coefficients : Double[]) : Bool {
    // NB: We don't currently use Any / Mapped for this, as we want to be
    //     able to short-circuit.
    for coefficient in coefficients {
        if AbsD(coefficient) >= tolerance {
            return true;
        }
    }
    return false;
}

function AnyOutsideToleranceCP(tolerance : Double, coefficients : ComplexPolar[]) : Bool {
    for coefficient in coefficients {
        if AbsComplexPolar(coefficient) > tolerance {
            return true;
        }
    }
    return false;
}

/// # Summary
/// Implementation step of arbitrary state preparation procedure.
function StatePreparationSBMComputeCoefficients(coefficients : ComplexPolar[]) : (Double[], Double[], ComplexPolar[]) {
    mutable disentanglingZ = [0.0, size = Length(coefficients) / 2];
    mutable disentanglingY = [0.0, size = Length(coefficients) / 2];
    mutable newCoefficients = Repeated(new ComplexPolar { Magnitude = 0.0, Argument = 0.0 }, Length(coefficients) / 2);

    for idxCoeff in 0..2..Length(coefficients) - 1 {
        let (rt, phi, theta) = BlochSphereCoordinates(coefficients[idxCoeff], coefficients[idxCoeff + 1]);
        disentanglingZ w/= idxCoeff / 2 <- 0.5 * phi;
        disentanglingY w/= idxCoeff / 2 <- 0.5 * theta;
        newCoefficients w/= idxCoeff / 2 <- rt;
    }

    return (disentanglingY, disentanglingZ, newCoefficients);
}

/// # Summary
/// Computes the Bloch sphere coordinates for a single-qubit state.
///
/// Given two complex numbers $a0, a1$ that represent the qubit state, computes coordinates
/// on the Bloch sphere such that
/// $a0 \ket{0} + a1 \ket{1} = r e^{it}(e^{-i \phi /2}\cos{(\theta/2)}\ket{0}+e^{i \phi /2}\sin{(\theta/2)}\ket{1})$.
///
/// # Input
/// ## a0
/// Complex coefficient of state $\ket{0}$.
/// ## a1
/// Complex coefficient of state $\ket{1}$.
///
/// # Output
/// A tuple containing `(ComplexPolar(r, t), phi, theta)`.
function BlochSphereCoordinates(a0 : ComplexPolar, a1 : ComplexPolar) : (ComplexPolar, Double, Double) {
    let abs0 = AbsComplexPolar(a0);
    let abs1 = AbsComplexPolar(a1);
    let arg0 = ArgComplexPolar(a0);
    let arg1 = ArgComplexPolar(a1);
    let r = Sqrt(abs0 * abs0 + abs1 * abs1);
    let t = 0.5 * (arg0 + arg1);
    let phi = arg1 - arg0;
    let theta = 2.0 * ArcTan2(abs1, abs0);
    return (new ComplexPolar { Magnitude = r, Argument = t }, phi, theta);
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
operation PrepareArbitraryStateD(coefficients : Double[], qubits : Qubit[]) : Unit is Adj + Ctl {
    ApproximatelyPrepareArbitraryStateD(0.0, coefficients, qubits);
}

/// # Summary
/// Given a set of coefficients and a little-endian encoded quantum register,
/// prepares an state on that register described by the given coefficients,
/// up to a given approximation tolerance.
///
/// # Description
/// This operation prepares an arbitrary quantum
/// state $\ket{\psi}$ with complex coefficients $r_j e^{i t_j}$ from
/// the $n$-qubit computational basis state $\ket{0 \cdots 0}$.
/// In particular, the action of this operation can be simulated by the
/// a unitary transformation $U$ which acts on the all-zeros state as
///
/// $$
/// \begin{align}
///     U\ket{0...0}
///         & = \ket{\psi} \\\\
///         & = \frac{
///                 \sum_{j=0}^{2^n-1} r_j e^{i t_j} \ket{j}
///             }{
///                 \sqrt{\sum_{j=0}^{2^n-1} |r_j|^2}
///             }.
/// \end{align}
/// $$
///
/// # Input
/// ## tolerance
/// The approximation tolerance to be used when preparing the given state.
///
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
/// Negative input coefficients $r_j < 0$ will be treated as though
/// positive with value $|r_j|$. `coefficients` will be padded with
/// elements $(r_j, t_j) = (0.0, 0.0)$ if fewer than $2^n$ are
/// specified.
///
/// # References
/// - [Synthesis of Quantum Logic Circuits](https://arxiv.org/abs/quant-ph/0406176)
///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
operation ApproximatelyPrepareArbitraryStateD(
    tolerance : Double,
    coefficients : Double[],
    qubits : Qubit[]
) : Unit is Adj + Ctl {
    let coefficientsAsComplexPolar = Mapped(c -> new ComplexPolar { Magnitude = AbsD(c), Argument = 0.0 }, coefficients);
    ApproximatelyPrepareArbitraryStateCP(tolerance, coefficientsAsComplexPolar, qubits);
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
