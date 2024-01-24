// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Unstable.StatePreparation {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Math;

    /// # Summary
    /// Given a set of coefficients and a little-endian encoded quantum register,
    /// prepares an state on that register described by the given coefficients.
    ///
    /// # Description
    /// This operation prepares an arbitrary quantum
    /// state $\ket{\psi}$ with coefficients $\alpha_j\ge 0$ from
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
    /// `coefficients` will be padded with
    /// elements $\alpha_j = 0.0$ if fewer than $2^n$ are specified.
    ///
    /// # Example
    /// The following snippet prepares the quantum state $\ket{\psi}=\sqrt{1/8}\ket{0}+\sqrt{7/8}\ket{2}$
    /// in the qubit register `qubitsLE`.
    /// ```qsharp
    /// let amplitudes = [Sqrt(0.125), 0.0, Sqrt(0.875), 0.0];
    /// use qubits = Qubit[2];
    /// let qubitsLE = LittleEndian(qubits);
    /// PreparePureStateD(amplitudes, qubitsLE);
    /// ```
    ///
    /// # References
    /// - Synthesis of Quantum Logic Circuits
    ///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
    ///   https://arxiv.org/abs/quant-ph/0406176
    ///
    /// # See Also
    /// - Microsoft.Quantum.Preparation.ApproximatelyPrepareArbitraryState
    operation PreparePureStateD(coefficients : Double[], qubits : Qubit[]) : Unit is Adj + Ctl {
        let coefficientsAsComplexPolar = Mapped(a -> ComplexAsComplexPolar(Complex(a, 0.0)), coefficients);
        ApproximatelyPreparePureStateCP(0.0, coefficientsAsComplexPolar, qubits);
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
    /// - Synthesis of Quantum Logic Circuits
    ///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
    ///   https://arxiv.org/abs/quant-ph/0406176
    operation ApproximatelyPreparePureStateCP(
        tolerance : Double,
        coefficients : ComplexPolar[],
        qubits : Qubit[]
    ) : Unit is Adj + Ctl {
        let op = (CompileApproximateArbitraryStatePreparation(tolerance, coefficients, Length(qubits)));
        op(qubits);
    }

    internal function CompileApproximateArbitraryStatePreparation(
        tolerance : Double,
        coefficients : ComplexPolar[],
        nQubits : Int
    ) : (Qubit[] => Unit is Adj + Ctl) {
        // pad coefficients at tail length to a power of 2.
        let coefficientsPadded = Padded(-2 ^ nQubits, ComplexPolar(0.0, 0.0), coefficients);
        let idxTarget = 0;
        let rngControl =
            // Determine what controls to apply to `op`.
            nQubits > 1
            ? (1 .. (nQubits - 1))
            | (1..0);
        let plan = ApproximatelyUnprepareArbitraryStatePlan(
            tolerance, coefficientsPadded, (rngControl, idxTarget)
        );
        let unprepare = BoundCA(plan);
        return Adjoint unprepare;
    }

    // TODO: Remove this.
    function BoundCA<'T> (operations : ('T => Unit is Adj + Ctl)[]) : ('T => Unit is Adj + Ctl) {
        return ApplyBoundCA(operations, _);
    }
    // TODO: Remove this.
    internal operation ApplyBoundCA<'T> (operations : ('T => Unit is Adj + Ctl)[], target : 'T)
    : Unit is Adj + Ctl {
        for op in operations {
            op(target);
        }
    }


    /// # Summary
    /// Implementation step of arbitrary state preparation procedure.
    ///
    /// # See Also
    /// - PrepareArbitraryState
    /// - Microsoft.Quantum.Canon.MultiplexPauli
    internal function ApproximatelyUnprepareArbitraryStatePlan(
        tolerance : Double, coefficients : ComplexPolar[],
        (rngControl : Range, idxTarget : Int)
    )
    : (Qubit[] => Unit is Adj + Ctl)[] {
        mutable plan = [];

        // For each 2D block, compute disentangling single-qubit rotation parameters
        let (disentanglingY, disentanglingZ, newCoefficients) = StatePreparationSBMComputeCoefficients(coefficients);
        if (AnyOutsideToleranceD(tolerance, disentanglingZ)) {
            set plan += [ApplyMultiplexStep(tolerance, disentanglingZ, PauliZ, (rngControl, idxTarget), _)];
        }
        if (AnyOutsideToleranceD(tolerance, disentanglingY)) {
            set plan += [ApplyMultiplexStep(tolerance, disentanglingY, PauliY, (rngControl, idxTarget), _)];
        }

        // target is now in |0> state up to the phase given by arg of newCoefficients.

        // Continue recursion while there are control qubits.
        if (IsRangeEmpty(rngControl)) {
            let (abs, arg) = newCoefficients[0]!;
            if (AbsD(arg) > tolerance) {
                set plan += [ApplyGlobalRotationStep(-1.0 * arg, idxTarget, _)];
            }
        } elif (AnyOutsideToleranceCP(tolerance, newCoefficients)) {
            let newControl = (RangeStart(rngControl) + 1)..RangeStep(rngControl)..RangeEnd(rngControl);
            let newTarget = RangeStart(rngControl);
            set plan += ApproximatelyUnprepareArbitraryStatePlan(tolerance, newCoefficients, (newControl, newTarget));
        }

        return plan;
    }


    internal operation ApplyMultiplexStep(
        tolerance : Double, disentangling : Double[], axis : Pauli,
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
    ///
    /// # See Also
    /// - Microsoft.Quantum.Canon.MultiplexPauli
    operation ApproximatelyMultiplexPauli(
        tolerance : Double,
        coefficients : Double[],
        pauli : Pauli,
        control : Qubit[],
        target : Qubit) : Unit is Adj + Ctl {
        if pauli == PauliZ {
            let op = ApproximatelyMultiplexZ(tolerance, coefficients, control, _);
            op(target);
        } elif pauli == PauliX {
            let op = ApproximatelyMultiplexPauli(tolerance, coefficients, PauliZ, control, _);
            ApplyWithCA(H, op, target);
        } elif pauli == PauliY {
            let op = ApproximatelyMultiplexPauli(tolerance, coefficients, PauliX, control, _);
            ApplyWithCA(Adjoint S, op, target);
        } elif pauli == PauliI {
            ApproximatelyApplyDiagonalUnitary(tolerance, coefficients, control);
        } else {
            fail $"MultiplexPauli failed. Invalid pauli {pauli}.";
        }
    }

    /// # Summary
    /// Given two operations, applies one as conjugated with the other.
    ///
    /// # Description
    /// Given two operations, respectively described by unitary operators $U$
    /// and $V$, applies them in the sequence $U^{\dagger} V U$. That is,
    /// this operation implements the unitary operator given by $V$ conjugated
    /// with $U$.
    ///
    /// # Input
    /// ## outerOperation
    /// The operation $U$ that should be used to conjugate $V$. Note that the
    /// outer operation $U$ needs to be adjointable, but does not
    /// need to be controllable.
    /// ## innerOperation
    /// The operation $V$ being conjugated.
    /// ## target
    /// The input to be provided to the outer and inner operations.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The target on which each of the inner and outer operations act.
    ///
    /// # Remarks
    /// The outer operation is always assumed to be adjointable, but does not
    /// need to be controllable in order for the combined operation to be
    /// controllable.
    ///
    /// # See Also
    /// - Microsoft.Quantum.Canon.ApplyWith
    /// - Microsoft.Quantum.Canon.ApplyWithA
    /// - Microsoft.Quantum.Canon.ApplyWithC
    operation ApplyWithCA<'T>(outerOperation : ('T => Unit is Adj), innerOperation : ('T => Unit is Adj + Ctl), target : 'T) : Unit {
        // TODO: Remove this.
        body (...) {
            outerOperation(target);
            innerOperation(target);
            Adjoint outerOperation(target);
        }

        adjoint auto;

        controlled (controlRegister, ...) {
            outerOperation(target);
            Controlled innerOperation(controlRegister, target);
            Adjoint outerOperation(target);
        }

        controlled adjoint auto;
    }


    /// # Summary
    /// Implementation step of arbitrary state preparation procedure.
    /// # See Also
    /// - Microsoft.Quantum.Preparation.PrepareArbitraryState
    internal function StatePreparationSBMComputeCoefficients(coefficients : ComplexPolar[]) : (Double[], Double[], ComplexPolar[]) {
        mutable disentanglingZ = [0.0, size = Length(coefficients) / 2];
        mutable disentanglingY = [0.0, size = Length(coefficients) / 2];
        mutable newCoefficients = [ComplexPolar(0.0, 0.0), size = Length(coefficients) / 2];

        for idxCoeff in 0 .. 2 .. Length(coefficients) - 1 {
            let (rt, phi, theta) = BlochSphereCoordinates(coefficients[idxCoeff], coefficients[idxCoeff + 1]);
            set disentanglingZ w/= idxCoeff / 2 <- 0.5 * phi;
            set disentanglingY w/= idxCoeff / 2 <- 0.5 * theta;
            set newCoefficients w/= idxCoeff / 2 <- rt;
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
    function BlochSphereCoordinates (a0 : ComplexPolar, a1 : ComplexPolar) : (ComplexPolar, Double, Double) {
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
    /// - Synthesis of Quantum Logic Circuits
    ///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
    ///   https://arxiv.org/abs/quant-ph/0406176
    ///
    /// # See Also
    /// - Microsoft.Quantum.Canon.ApplyDiagonalUnitary
    operation ApproximatelyApplyDiagonalUnitary(tolerance : Double, coefficients : Double[], qubits : Qubit[])
    : Unit is Adj + Ctl {
        if IsEmpty(qubits) {
            fail "operation ApplyDiagonalUnitary -- Number of qubits must be greater than 0.";
        }

        // pad coefficients length at tail to a power of 2.
        let coefficientsPadded = Padded(-2 ^ Length(qubits), 0.0, coefficients);

        // Compute new coefficients.
        let (coefficients0, coefficients1) = MultiplexZCoefficients(coefficientsPadded);
        ApproximatelyMultiplexZ(tolerance,coefficients1, Most(qubits), Tail(qubits));

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
    /// - Synthesis of Quantum Logic Circuits
    ///   Vivek V. Shende, Stephen S. Bullock, Igor L. Markov
    ///   https://arxiv.org/abs/quant-ph/0406176
    ///
    /// # See Also
    /// - Microsoft.Quantum.Canon.MultiplexZ
    operation ApproximatelyMultiplexZ(
        tolerance : Double,
        coefficients : Double[],
        control : Qubit[],
        target : Qubit) : Unit is Adj + Ctl {

        body (...) {
            // pad coefficients length at tail to a power of 2.
            let coefficientsPadded = Padded(-2 ^ Length(control), 0.0, coefficients);

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
            let coefficientsPadded = Padded(2 ^ (Length(control) + 1), 0.0, Padded(-2 ^ Length(control), 0.0, coefficients));
            let (coefficients0, coefficients1) = MultiplexZCoefficients(coefficientsPadded);
            ApproximatelyMultiplexZ(tolerance,coefficients0, control, target);
            if AnyOutsideToleranceD(tolerance,coefficients1) {
                within {
                    Controlled X(controlRegister, target);
                } apply {
                    ApproximatelyMultiplexZ(tolerance,coefficients1, control, target);
                }
            }
        }
    }

    /// # Summary
    /// Implementation step of multiply-controlled Z rotations.
    /// # See Also
    /// - Microsoft.Quantum.Canon.MultiplexZ
    internal function MultiplexZCoefficients(coefficients : Double[]) : (Double[], Double[]) {
        let newCoefficientsLength = Length(coefficients) / 2;
        mutable coefficients0 = [0.0, size = newCoefficientsLength];
        mutable coefficients1 = [0.0, size = newCoefficientsLength];

        for idxCoeff in 0 .. newCoefficientsLength - 1 {
            set coefficients0 w/= idxCoeff <- 0.5 * (coefficients[idxCoeff] + coefficients[idxCoeff + newCoefficientsLength]);
            set coefficients1 w/= idxCoeff <- 0.5 * (coefficients[idxCoeff] - coefficients[idxCoeff + newCoefficientsLength]);
        }

        return (coefficients0, coefficients1);
    }

    internal function AnyOutsideToleranceD(tolerance : Double, coefficients : Double[]) : Bool {
        for coefficient in coefficients {
            if AbsD(coefficient) >= tolerance { // TODO: Why is this >= and not > ???
                return true;
            }
        }
        return false;
    }

    internal function AnyOutsideToleranceCP(tolerance : Double, coefficients : ComplexPolar[]) : Bool {
        for coefficient in coefficients {
            if AbsComplexPolar(coefficient) > tolerance {
                return true;
            }
        }
        return false;
    }

    internal operation ApplyGlobalRotationStep(
        angle : Double, idxTarget : Int, register : Qubit[]
    ) : Unit is Adj + Ctl {
        Exp([PauliI], angle, [register[idxTarget]]);
    }

    /// # Summary
    /// Returns true if and only if input range is empty.
    ///
    /// # Input
    /// ## rng
    /// Any range
    ///
    /// # Output
    /// True, if and only if `rng` is empty
    ///
    /// # Remark
    /// This function needs to check at most one range index
    /// to determine whether the range is empty.
    function IsRangeEmpty(rng : Range) : Bool {
        // TODO: Consider moving and making public
        for idx in rng {
            return false;
        }
        return true;
    }

}

