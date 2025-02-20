// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

export
    PreparePureStateD,
    ApproximatelyPreparePureStateCP,
    PrepareUniformSuperposition;

import
    Std.Diagnostics.Fact,
    Std.Convert.ComplexAsComplexPolar,
    Std.Convert.IntAsDouble,
    Std.Arithmetic.ApplyIfGreaterLE,
    Std.Math.*,
    Std.Arrays.*;

/// # Summary
/// Given a set of coefficients and a big-endian quantum register,
/// prepares a state on that register described by the given coefficients.
///
/// # Description
/// This operation prepares an arbitrary quantum
/// state |𝜓⟩ with coefficients 𝑎ⱼ from
/// the n-qubit computational basis state |0...0⟩.
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
/// Array of up to 2ⁿ real coefficients. The j-th coefficient
/// indexes the number state |j⟩ encoded in big-endian format.
///
/// ## qubits
/// Qubit register encoding number states in a big-endian format. This is
/// expected to be initialized in the computational basis state |0...0⟩.
///
/// # Remarks
/// `coefficients` will be normalized and padded with
/// elements 𝑎ⱼ = 0.0 if fewer than 2ⁿ are specified.
///
/// # Example
/// The following snippet prepares the quantum state |𝜓⟩=√(1/8)|0⟩+√(7/8)|2⟩=√(1/8)|00⟩+√(7/8)|10⟩
/// in the qubit register `qubits`.
/// ```qsharp
/// let amplitudes = [Sqrt(0.125), 0.0, Sqrt(0.875), 0.0];
/// use qubits = Qubit[2];
/// PreparePureStateD(amplitudes, qubits);
/// ```
///
/// # References
/// - [arXiv:quant-ph/0406176](https://arxiv.org/abs/quant-ph/0406176)
///   "Synthesis of Quantum Logic Circuits",
///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
///
/// # See Also
/// - Std.StatePreparation.ApproximatelyPreparePureStateCP
operation PreparePureStateD(coefficients : Double[], qubits : Qubit[]) : Unit is Adj + Ctl {
    let coefficientsAsComplexPolar = Mapped(a -> ComplexAsComplexPolar(Complex(a, 0.0)), coefficients);
    ApproximatelyPreparePureStateCP(0.0, coefficientsAsComplexPolar, qubits);
}

/// # Summary
/// Given a set of coefficients and a big-endian quantum register,
/// prepares a state on that register described by the given coefficients,
/// up to a given approximation tolerance.
///
/// # Description
/// This operation prepares an arbitrary quantum
/// state |𝜓⟩ with complex coefficients rⱼ·𝒆^(𝒊·tⱼ) from
/// the n-qubit computational basis state |0...0⟩.
/// In particular, the action of this operation can be simulated by the
/// a unitary transformation U which acts on the all-zeros state as
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
/// Array of up to 2ⁿ complex coefficients represented by their
/// absolute value and phase (rⱼ, tⱼ). The j-th coefficient
/// indexes the number state |j⟩ encoded in a big-endian format.
///
/// ## qubits
/// Qubit register encoding number states in a big-endian format. This is
/// expected to be initialized in the computational basis state
/// |0...0⟩.
///
/// # Remarks
/// `coefficients` will be padded with
/// elements (rⱼ, tⱼ) = (0.0, 0.0) if fewer than 2ⁿ are
/// specified.
///
/// # References
/// - [arXiv:quant-ph/0406176](https://arxiv.org/abs/quant-ph/0406176)
///   "Synthesis of Quantum Logic Circuits",
///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
operation ApproximatelyPreparePureStateCP(
    tolerance : Double,
    coefficients : ComplexPolar[],
    qubits : Qubit[]
) : Unit is Adj + Ctl {

    let nQubits = Length(qubits);
    // pad coefficients at tail length to a power of 2.
    let coefficientsPadded = Padded(-2^nQubits, ComplexPolar(0.0, 0.0), coefficients);
    let idxTarget = 0;
    // Determine what controls to apply
    let rngControl = nQubits > 1 ? (1..(nQubits - 1)) | (1..0);
    // Note we use the reversed qubits array to get the endianness ordering that we expect
    // when corresponding qubit state to state vector index.
    Adjoint ApproximatelyUnprepareArbitraryState(
        tolerance,
        coefficientsPadded,
        rngControl,
        idxTarget,
        Reversed(qubits)
    );
}

/// # Summary
/// Implementation step of arbitrary state preparation procedure.
operation ApproximatelyUnprepareArbitraryState(
    tolerance : Double,
    coefficients : ComplexPolar[],
    rngControl : Range,
    idxTarget : Int,
    register : Qubit[]
) : Unit is Adj + Ctl {

    // For each 2D block, compute disentangling single-qubit rotation parameters
    let (disentanglingY, disentanglingZ, newCoefficients) = StatePreparationSBMComputeCoefficients(coefficients);
    if (AnyOutsideToleranceD(tolerance, disentanglingZ)) {
        ApproximatelyMultiplexPauli(tolerance, disentanglingZ, PauliZ, register[rngControl], register[idxTarget]);

    }
    if (AnyOutsideToleranceD(tolerance, disentanglingY)) {
        ApproximatelyMultiplexPauli(tolerance, disentanglingY, PauliY, register[rngControl], register[idxTarget]);
    }
    // target is now in |0> state up to the phase given by arg of newCoefficients.

    // Continue recursion while there are control qubits.
    if (IsRangeEmpty(rngControl)) {
        let (abs, arg) = newCoefficients[0]!;
        if (AbsD(arg) > tolerance) {
            Exp([PauliI], -1.0 * arg, [register[idxTarget]]);
        }
    } elif (Any(c -> AbsComplexPolar(c) > tolerance, newCoefficients)) {
        // Some coefficients are outside tolerance
        let newControl = (RangeStart(rngControl) + 1)..RangeStep(rngControl)..RangeEnd(rngControl);
        let newTarget = RangeStart(rngControl);
        ApproximatelyUnprepareArbitraryState(tolerance, newCoefficients, newControl, newTarget, register);
    }
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
operation ApproximatelyMultiplexPauli(
    tolerance : Double,
    coefficients : Double[],
    pauli : Pauli,
    control : Qubit[],
    target : Qubit
) : Unit is Adj + Ctl {

    if pauli == PauliZ {
        ApproximatelyMultiplexZ(tolerance, coefficients, control, target);
    } elif pauli == PauliX {
        within {
            H(target);
        } apply {
            ApproximatelyMultiplexPauli(tolerance, coefficients, PauliZ, control, target);
        }
    } elif pauli == PauliY {
        within {
            Adjoint S(target);
        } apply {
            ApproximatelyMultiplexPauli(tolerance, coefficients, PauliX, control, target);
        }
    } else {
        fail $"MultiplexPauli failed. Invalid pauli {pauli}.";
    }
}

/// # Summary
/// Implementation step of arbitrary state preparation procedure.
function StatePreparationSBMComputeCoefficients(
    coefficients : ComplexPolar[]
) : (Double[], Double[], ComplexPolar[]) {

    mutable disentanglingZ = [];
    mutable disentanglingY = [];
    mutable newCoefficients = [];

    for idxCoeff in 0..2..Length(coefficients) - 1 {
        let (rt, phi, theta) = BlochSphereCoordinates(coefficients[idxCoeff], coefficients[idxCoeff + 1]);
        set disentanglingZ += [0.5 * phi];
        set disentanglingY += [0.5 * theta];
        set newCoefficients += [rt];
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
function BlochSphereCoordinates(
    a0 : ComplexPolar,
    a1 : ComplexPolar
) : (ComplexPolar, Double, Double) {

    let abs0 = AbsComplexPolar(a0);
    let abs1 = AbsComplexPolar(a1);
    let arg0 = ArgComplexPolar(a0);
    let arg1 = ArgComplexPolar(a1);
    let r = Sqrt(abs0 * abs0 + abs1 * abs1);
    let t = 0.5 * (arg0 + arg1);
    let phi = arg1 - arg0;
    let theta = 2.0 * ArcTan2(abs1, abs0);
    return (ComplexPolar(r, t), phi, theta);
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
/// - [arXiv:quant-ph/0406176](https://arxiv.org/abs/quant-ph/0406176)
///   "Synthesis of Quantum Logic Circuits",
///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
operation ApproximatelyMultiplexZ(
    tolerance : Double,
    coefficients : Double[],
    control : Qubit[],
    target : Qubit
) : Unit is Adj + Ctl {

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
    mutable coefficients0 = [];
    mutable coefficients1 = [];

    for idxCoeff in 0..newCoefficientsLength - 1 {
        set coefficients0 += [0.5 * (coefficients[idxCoeff] + coefficients[idxCoeff + newCoefficientsLength])];
        set coefficients1 += [0.5 * (coefficients[idxCoeff] - coefficients[idxCoeff + newCoefficientsLength])];
    }

    return (coefficients0, coefficients1);
}

function AnyOutsideToleranceD(tolerance : Double, coefficients : Double[]) : Bool {
    // NOTE: This function is not used as the only recursion termination condition
    // only to determine if the multiplex step needs to be applied.
    // For tolerance 0.0 it is always applied due to >= comparison.
    Any(coefficient -> AbsD(coefficient) >= tolerance, coefficients)
}

/// # Summary
/// Prepares a uniform superposition of states that represent integers 0 through
/// `nStates - 1` in a little-endian `qubits` register.
///
/// # Description
/// Given an input state $\ket{0\cdots 0}$ this operation prepares
/// a uniform superposition of all number states $0$ to $M-1$. In other words,
/// $$
/// \begin{align}
///     \ket{0} \mapsto \frac{1}{\sqrt{M}} \sum_{j=0}^{M - 1} \ket{j}
/// \end{align}
/// $$
///
/// The operation is adjointable, but requires that `qubits` register is in a
/// uniform superposition over the first `nStates` basis states in that case.
///
/// # Input
/// ## nStates
/// The number of states in the uniform superposition to be prepared.
/// ## register
/// The little-endian qubit register to store the prepared state.
/// It is assumed to be initialized in the zero state $\ket{0\cdots 0}$.
/// This register must be long enough to store the number $M-1$, meaning that
/// $2^{Length(qubits)} >= M$.
///
/// # Example
/// ```qsharp
///    use qs = Qubit[4];
///    PrepareUniformSuperposition(3, qs);
///    DumpRegister(qs); // The state is (|0000>+|0100>+|1000>)/√3
///    ResetAll(qs);
/// ```
operation PrepareUniformSuperposition(nStates : Int, qubits : Qubit[]) : Unit is Adj + Ctl {
    Fact (nStates > 0, "Number of basis states must be positive.");
    let nQubits = BitSizeI(nStates - 1);
    Fact(nQubits <= Length(qubits), $"Qubit register is too short to prepare {nStates} states.");
    let completeStateCount = 2^nQubits;

    if nStates == completeStateCount {
        // Superposition over all states involving nQubits
        for i in 0..nQubits-1 {
            H(qubits[i]);
        }
    } else {
        use flagQubit = Qubit[3];
        let targetQubits = qubits[0..nQubits - 1];
        let register = flagQubit + targetQubits;
        let stateOracle = PrepareUniformSuperpositionOracle(nStates, nQubits, 0, _);

        let phases = ([0.0, PI()], [PI(), 0.0]);

        ObliviousAmplitudeAmplificationFromStatePreparation(
            phases,
            stateOracle,
            (qs0, qs1) => I(qs0[0]),
            0
        )(register, []);

        ApplyToEachCA(X, flagQubit);
    }
}

operation PrepareUniformSuperpositionOracle(nIndices : Int, nQubits : Int, idxFlag : Int, qubits : Qubit[]) : Unit is Adj + Ctl {
    let targetQubits = qubits[3...];
    let flagQubit = qubits[0];
    let auxillaryQubits = qubits[1..2];
    let theta = ArcSin(Sqrt(IntAsDouble(2^nQubits) / IntAsDouble(nIndices)) * Sin(PI() / 6.0));

    ApplyToEachCA(H, targetQubits);
    use compareQubits = Qubit[nQubits] {
        within {
            ApplyXorInPlace(nIndices - 1, compareQubits);
        } apply {
            ApplyIfGreaterLE(X, targetQubits, compareQubits, auxillaryQubits[0]);
            X(auxillaryQubits[0]);
        }
    }
    Ry(2.0 * theta, auxillaryQubits[1]);
    (Controlled X)(auxillaryQubits, flagQubit);
}

function ObliviousAmplitudeAmplificationFromStatePreparation(
    phases : (Double[], Double[]),
    startStateOracle : Qubit[] => Unit is Adj + Ctl,
    signalOracle : (Qubit[], Qubit[]) => Unit is Adj + Ctl,
    idxFlagQubit : Int
) : (Qubit[], Qubit[]) => Unit is Adj + Ctl {
    let startStateReflection = (phase, qs) => {
        within {
            ApplyToEachCA(X, qs);
        } apply {
            Controlled R1(Rest(qs), (phase, qs[0]));
        }
    };
    let targetStateReflection = (phase, qs) => R1(phase, qs[idxFlagQubit]);
    let obliviousSignalOracle = (qs0, qs1) => { startStateOracle(qs0); signalOracle(qs0, qs1); };
    return ObliviousAmplitudeAmplificationFromPartialReflections(
        phases,
        startStateReflection,
        targetStateReflection,
        obliviousSignalOracle
    );
}


function ObliviousAmplitudeAmplificationFromPartialReflections(
    phases : (Double[], Double[]),
    startStateReflection : (Double, Qubit[]) => Unit is Adj + Ctl,
    targetStateReflection : (Double, Qubit[]) => Unit is Adj + Ctl,
    signalOracle : (Qubit[], Qubit[]) => Unit is Adj + Ctl
) : (Qubit[], Qubit[]) => Unit is Adj + Ctl {
    return ApplyObliviousAmplitudeAmplification(
        phases,
        startStateReflection,
        targetStateReflection,
        signalOracle,
        _,
        _
    );
}

/// # Summary
/// Oblivious amplitude amplification by specifying partial reflections.
///
/// # Description
///
/// Given a particular auxiliary start state $\ket{\text{start}}_a$, a
/// particular auxiliary target state $\ket{\text{target}}_a$, and any
/// system state $\ket{\psi}_s$, suppose that
/// $$
/// \begin{align}
///     O\ket{\text{start}}_a\ket{\psi}_s = \lambda\ket{\text{target}}_a U \ket{\psi}_s + \sqrt{1-|\lambda|^2}\ket{\text{target}^\perp}_a
/// \end{align}
/// $$
/// for some unitary $U$.
/// By a sequence of reflections about the start and target states on the
/// auxiliary register interleaved by applications of `signalOracle` and its
/// adjoint, the success probability of applying $U$ may be altered.
///
/// In most cases, `auxiliaryRegister` is initialized in the state $\ket{\text{start}}_a$.
///
/// # Input
/// ## phases
/// Phases of partial reflections
/// ## startStateReflection
/// Reflection operator about start state of auxiliary register
/// ## targetStateReflection
/// Reflection operator about target state of auxiliary register
/// ## signalOracle
/// Unitary oracle $O$ that acts jointly on the
/// auxiliary and system registers.
/// ## auxiliaryRegister
/// Auxiliary register
/// ## systemRegister
/// System register
///
/// # References
/// - See [*D.W. Berry, A.M. Childs, R. Cleve, R. Kothari, R.D. Somma*](https://arxiv.org/abs/1312.1414)
/// for the standard version.
/// - See [*G.H. Low, I.L. Chuang*](https://arxiv.org/abs/1610.06546)
/// for a generalization to partial reflections.
operation ApplyObliviousAmplitudeAmplification(
    phases : (Double[], Double[]),
    startStateReflection : (Double, Qubit[]) => Unit is Adj + Ctl,
    targetStateReflection : (Double, Qubit[]) => Unit is Adj + Ctl,
    signalOracle : (Qubit[], Qubit[]) => Unit is Adj + Ctl,
    auxiliaryRegister : Qubit[],
    systemRegister : Qubit[]
) : Unit is Adj + Ctl {
    let (aboutStart, aboutTarget) = phases;
    Fact(Length(aboutStart) == Length(aboutTarget), "number of phases about start and target state must be equal");
    let numPhases = Length(aboutStart);

    for idx in 0..numPhases-1 {
        if aboutStart[idx] != 0.0 {
            startStateReflection(aboutStart[idx], auxiliaryRegister);
        }

        if aboutTarget[idx] != 0.0 {
            // In the last iteration we do not need to apply `Adjoint signalOracle`
            if idx == numPhases - 1 {
                signalOracle(auxiliaryRegister, systemRegister);
                targetStateReflection(aboutTarget[idx], auxiliaryRegister);
            } else {
                within {
                    signalOracle(auxiliaryRegister, systemRegister);
                } apply {
                    targetStateReflection(aboutTarget[idx], auxiliaryRegister);
                }
            }
        }
    }

    // We do need one more `signalOracle` call, if the last phase about the target state was 0.0
    if numPhases == 0 or aboutTarget[numPhases - 1] == 0.0 {
        signalOracle(auxiliaryRegister, systemRegister);
    }
}

