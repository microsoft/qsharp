// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Intrinsic {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Core;
    open Microsoft.Quantum.Math;
    open QIR.Intrinsic;

    /// # Summary
    /// Applies the doubly controlled–NOT (CCNOT) gate to three qubits.
    ///
    /// # Input
    /// ## control1
    /// First control qubit for the CCNOT gate.
    /// ## control2
    /// Second control qubit for the CCNOT gate.
    /// ## target
    /// Target qubit for the CCNOT gate.
    ///
    /// # Remarks
    /// Equivalent to:
    /// ```qsharp
    /// Controlled X([control1, control2], target);
    /// ```
    operation CCNOT(control1 : Qubit, control2 : Qubit, target : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__ccx__body(control1, control2, target);
        }
        controlled (ctls, ...) {
            Controlled X(ctls + [control1, control2], target);
        }
        adjoint self;
    }

    /// # Summary
    /// Applies the controlled-NOT (CNOT) gate to a pair of qubits.
    ///
    /// # Description
    /// \begin{align}
    ///     \operatorname{CNOT} \mathrel{:=}
    ///     \begin{bmatrix}
    ///         1 & 0 & 0 & 0 \\\\
    ///         0 & 1 & 0 & 0 \\\\
    ///         0 & 0 & 0 & 1 \\\\
    ///         0 & 0 & 1 & 0
    ///     \end{bmatrix},
    /// \end{align}
    ///
    /// where rows and columns are ordered as in the quantum concepts guide.
    ///
    /// # Input
    /// ## control
    /// Control qubit for the CNOT gate.
    /// ## target
    /// Target qubit for the CNOT gate.
    ///
    /// # Remarks
    /// Equivalent to:
    /// ```qsharp
    /// Controlled X([control], target);
    /// ```
    operation CNOT(control : Qubit, target : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__cx__body(control, target);
        }
        controlled (ctls, ...) {
            Controlled X(ctls + [control], target);
        }
        adjoint self;
    }

    /// # Summary
    /// Applies the Hadamard transformation to a single qubit.
    ///
    /// # Description
    /// \begin{align}
    ///     H \mathrel{:=}
    ///     \frac{1}{\sqrt{2}}
    ///     \begin{bmatrix}
    ///         1 & 1 \\\\
    ///         1 & -1
    ///     \end{bmatrix}.
    /// \end{align}
    ///
    /// # Input
    /// ## qubit
    /// Qubit to which the gate should be applied.
    operation H(qubit : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__h__body(qubit);
        }
        controlled (ctls, ...) {
            if Length(ctls) == 0 {
                __quantum__qis__h__body(qubit);
            }
            elif Length(ctls) == 1 {
                CH(ctls[0], qubit);
            }
            elif Length(ctls) == 2 {
                CCH(ctls[0], ctls[1], qubit);
            }
            else {
                use aux = Qubit[Length(ctls) - 1 - (Length(ctls) % 2)];
                within {
                    CollectControls(ctls, aux, 0);
                }
                apply {
                    if Length(ctls) % 2 != 0 {
                        CCH(ctls[Length(ctls) - 1], aux[Length(ctls) - 3], qubit);
                    }
                    else {
                        CCH(aux[Length(ctls) - 3], aux[Length(ctls) - 4], qubit);
                    }
                }
            }
        }
        adjoint self;
    }

    /// # Summary
    /// Performs the identity operation (no-op) on a single qubit.
    ///
    /// # Remarks
    /// This is a no-op. It is provided for completeness and because
    /// sometimes it is useful to call the identity in an algorithm or to pass it as a parameter.
    operation I(target : Qubit) : Unit is Adj + Ctl {
        body ... { }
        adjoint self;
    }

    /// # Summary
    /// Performs a measurement of a single qubit in the
    /// Pauli $Z$ basis.
    ///
    /// # Description
    /// The output result is given by
    /// the distribution
    /// \begin{align}
    ///     \Pr(\texttt{Zero} | \ket{\psi}) =
    ///         \braket{\psi | 0} \braket{0 | \psi}.
    /// \end{align}
    ///
    /// # Input
    /// ## qubit
    /// Qubit to be measured.
    ///
    /// # Output
    /// `Zero` if the $+1$ eigenvalue is observed, and `One` if
    /// the $-1$ eigenvalue is observed.
    ///
    /// # Remarks
    /// Equivalent to:
    /// ```qsharp
    /// Measure([PauliZ], [qubit]);
    /// ```
    operation M(qubit : Qubit) : Result {
        __quantum__qis__m__body(qubit)
    }

    /// # Summary
    /// Performs a joint measurement of one or more qubits in the
    /// specified Pauli bases.
    ///
    /// # Description
    /// The output result is given by the distribution:
    /// \begin{align}
    ///     \Pr(\texttt{Zero} | \ket{\psi}) =
    ///         \frac12 \braket{
    ///             \psi \mid|
    ///             \left(
    ///                 \boldone + P_0 \otimes P_1 \otimes \cdots \otimes P_{N-1}
    ///             \right) \mid|
    ///             \psi
    ///         },
    /// \end{align}
    /// where $P_i$ is the $i$th element of `bases`, and where
    /// $N = \texttt{Length}(\texttt{bases})$.
    /// That is, measurement returns a `Result` $d$ such that the eigenvalue of the
    /// observed measurement effect is $(-1)^d$.
    ///
    /// # Input
    /// ## bases
    /// Array of single-qubit Pauli values indicating the tensor product
    /// factors on each qubit.
    /// ## qubits
    /// Register of qubits to be measured.
    ///
    /// # Output
    /// `Zero` if the $+1$ eigenvalue is observed, and `One` if
    /// the $-1$ eigenvalue is observed.
    ///
    /// # Remarks
    /// If the basis array and qubit array are different lengths, then the
    /// operation will fail.
    operation Measure(bases : Pauli[], qubits : Qubit[]) : Result {
        mutable res = Zero;
        if Length(bases) != Length(qubits) {
            fail "Arrays 'bases' and 'qubits' must be of the same length.";
        }
        if Length(bases) == 1 {
            within {
                MapPauli(qubits[0], PauliZ, bases[0]);
            }
            apply {
                set res = __quantum__qis__m__body(qubits[0]);
            }
        }
        else {
            use aux = Qubit();
            within {
                H(aux);
            }
            apply {
                for i in 0..Length(bases)-1 {
                    EntangleForJointMeasure(bases[i], aux, qubits[i]);
                }
            }
            set res = __quantum__qis__mresetz__body(aux);
        }
        res
    }

    /// # Summary
    /// Applies a rotation about the given Pauli axis.
    ///
    /// # Description
    /// \begin{align}
    ///     R_{\mu}(\theta) \mathrel{:=}
    ///     e^{-i \theta \sigma_{\mu} / 2},
    /// \end{align}
    /// where $\mu \in \{I, X, Y, Z\}$.
    ///
    /// # Input
    /// ## pauli
    /// Pauli operator ($\mu$) to be exponentiated to form the rotation.
    /// ## theta
    /// Angle in radians about which the qubit is to be rotated.
    /// ## qubit
    /// Qubit to which the gate should be applied.
    ///
    /// # Remarks
    /// When called with `pauli = PauliI`, this operation applies
    /// a *global phase*. This phase can be significant
    /// when used with the `Controlled` functor.
    operation R(pauli : Pauli, theta : Double, qubit : Qubit) : Unit is Adj + Ctl {
        if (pauli == PauliX) {
            Rx(theta, qubit);
        }
        elif (pauli == PauliY) {
            Ry(theta, qubit);
        }
        elif (pauli == PauliZ) {
            Rz(theta, qubit);
        }
        else { // PauliI
            ApplyGlobalPhase( - theta / 2.0 );
        }
    }

    /// # Summary
    /// Applies a rotation about the $\ket{1}$ state by a given angle.
    ///
    /// # Description
    /// \begin{align}
    ///     R_1(\theta) \mathrel{:=}
    ///     \operatorname{diag}(1, e^{i\theta}).
    /// \end{align}
    ///
    /// # Input
    /// ## theta
    /// Angle about which the qubit is to be rotated.
    /// ## qubit
    /// Qubit to which the gate should be applied.
    ///
    /// # Remarks
    /// Equivalent to:
    /// ```qsharp
    /// R(PauliZ, theta, qubit);
    /// R(PauliI, -theta, qubit);
    /// ```
    operation R1(theta : Double, qubit : Qubit) : Unit is Adj + Ctl {
        body ... {
            Rz(theta, qubit);
        }
        controlled (ctls, ...) {
            if Length(ctls) == 0 {
                Rz(theta, qubit);
            }
            elif Length(ctls) == 1 {
                CR1(theta, ctls[0], qubit);
            }
            else {
                use aux = Qubit[Length(ctls) - 1];
                within {
                    CollectControls(ctls, aux, 0);
                    AdjustForSingleControl(ctls, aux);
                }
                apply {
                    CR1(theta, aux[Length(ctls) - 2], qubit);
                }
            }
        }
    }

    /// # Summary
    /// Applies a rotation about the $\ket{1}$ state by an angle specified
    /// as a dyadic fraction.
    ///
    /// # Description
    /// \begin{align}
    ///     R_1(n, k) \mathrel{:=}
    ///     \operatorname{diag}(1, e^{i \pi k / 2^n}).
    /// \end{align}
    ///
    /// > [!WARNING]
    /// > This operation uses the **opposite** sign convention from
    /// > @"microsoft.quantum.intrinsic.r", and does not include the
    /// > factor of $1/ 2$ included by @"microsoft.quantum.intrinsic.r1".
    ///
    /// # Input
    /// ## numerator
    /// Numerator in the dyadic fraction representation of the angle
    /// by which the qubit is to be rotated. This angle is expressed in radians.
    /// ## power
    /// Power of two specifying the denominator of the angle by which
    /// the qubit is to be rotated. This angle is expressed in radians.
    /// ## qubit
    /// Qubit to which the gate should be applied.
    ///
    /// # Remarks
    /// Equivalent to:
    /// ```qsharp
    /// RFrac(PauliZ, -numerator, denominator + 1, qubit);
    /// RFrac(PauliI, numerator, denominator + 1, qubit);
    /// ```
    operation R1Frac(numerator : Int, power : Int, qubit : Qubit) : Unit is Adj + Ctl {
        RFrac(PauliZ, -numerator, power + 1, qubit);
        RFrac(PauliI, numerator, power + 1, qubit);
    }

    /// # Summary
    /// Given a single qubit, measures it and ensures it is in the |0⟩ state
    /// such that it can be safely released.
    ///
    /// # Input
    /// ## qubit
    /// The qubit whose state is to be reset to $\ket{0}$.
    operation Reset(qubit : Qubit) : Unit {
        __quantum__qis__reset__body(qubit);
    }

    /// # Summary
    /// Given an array of qubits, measure them and ensure they are in the |0⟩ state
    /// such that they can be safely released.
    ///
    /// # Input
    /// ## qubits
    /// An array of qubits whose states are to be reset to $\ket{0}$.
    operation ResetAll(qubits : Qubit[]) : Unit {
        for q in qubits {
            Reset(q);
        }
    }

    /// # Summary
    /// Applies a rotation about the given Pauli axis by an angle specified
    /// as a dyadic fraction.
    ///
    /// # Description
    /// \begin{align}
    ///     R_{\mu}(n, k) \mathrel{:=}
    ///     e^{i \pi n \sigma_{\mu} / 2^k},
    /// \end{align}
    /// where $\mu \in \{I, X, Y, Z\}$.
    ///
    /// > [!WARNING]
    /// > This operation uses the **opposite** sign convention from
    /// > @"microsoft.quantum.intrinsic.r".
    ///
    /// # Input
    /// ## pauli
    /// Pauli operator to be exponentiated to form the rotation.
    /// ## numerator
    /// Numerator in the dyadic fraction representation of the angle
    /// by which the qubit is to be rotated. This angle is expressed in radians.
    /// ## power
    /// Power of two specifying the denominator of the angle by which
    /// the qubit is to be rotated. This angle is expressed in radians.
    /// ## qubit
    /// Qubit to which the gate should be applied.
    ///
    /// # Remarks
    /// Equivalent to:
    /// ```qsharp
    /// // PI() is a Q# function that returns an approximation of π.
    /// R(pauli, -PI() * IntAsDouble(numerator) / IntAsDouble(2 ^ (power - 1)), qubit);
    /// ```
    operation RFrac(pauli : Pauli, numerator : Int, power : Int, qubit : Qubit) : Unit is Adj + Ctl {
        // Note that power must be converted to a double and used with 2.0 instead of 2 to allow for
        // negative exponents that result in a fractional denominator.
        let angle = ((-2.0 * PI()) * IntAsDouble(numerator)) / (2.0 ^ IntAsDouble(power));
        R(pauli, angle, qubit);
    }

    /// # Summary
    /// Applies a rotation about the $x$-axis by a given angle.
    ///
    /// # Description
    /// \begin{align}
    ///     R_x(\theta) \mathrel{:=}
    ///     e^{-i \theta \sigma_x / 2} = 
    ///     \begin{bmatrix}
    ///         \cos \frac{\theta}{2} & -i\sin \frac{\theta}{2}  \\\\
    ///         -i\sin \frac{\theta}{2} & \cos \frac{\theta}{2}
    ///     \end{bmatrix}.
    /// \end{align}
    ///
    /// # Input
    /// ## theta
    /// Angle about which the qubit is to be rotated.
    /// ## qubit
    /// Qubit to which the gate should be applied.
    ///
    /// # Remarks
    /// Equivalent to:
    /// ```qsharp
    /// R(PauliX, theta, qubit);
    /// ```
    operation Rx(theta : Double, qubit : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__rx__body(theta, qubit);
        }
        controlled (ctls, ...) {
            if Length(ctls) == 0 {
                __quantum__qis__rx__body(theta, qubit);
            }
            else {
                within {
                    MapPauli(qubit, PauliZ, PauliX);
                }
                apply {
                    Controlled Rz(ctls, (theta, qubit));
                }
            }
        }
        adjoint ... {
            Rx(-theta, qubit);
        }
    }

    /// # Summary
    /// Applies the two qubit Ising $XX$ rotation gate.
    ///
    /// # Description
    /// \begin{align}
    ///     R_xx(\theta) \mathrel{:=}
    ///     \begin{bmatrix}
    ///         \cos \theta & 0 & 0 & -i\sin \theta  \\\\
    ///         0 & \cos \theta & -i\sin \theta & 0  \\\\
    ///         0 & -i\sin \theta & \cos \theta & 0  \\\\
    ///         -i\sin \theta & 0 & 0 & \cos \theta
    ///     \end{bmatrix}.
    /// \end{align}
    ///
    /// # Input
    /// ## theta
    /// The angle about which the qubits are rotated.
    /// ## qubit0
    /// The first qubit input to the gate.
    /// ## qubit1
    /// The second qubit input to the gate.
    operation Rxx(theta : Double, qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__rxx__body(theta, qubit0, qubit1);
        }
        controlled (ctls, ...) {
            if Length(ctls) == 0 {
                __quantum__qis__rxx__body(theta, qubit0, qubit1);
            }
            elif Length(ctls) == 1 {
                CRxx(ctls[0], theta, qubit0, qubit1);
            }
            else {
                use aux = Qubit[Length(ctls) - 1];
                within {
                    CollectControls(ctls, aux, 0);
                    AdjustForSingleControl(ctls, aux);
                }
                apply {
                    CRxx(aux[Length(ctls) - 2], theta, qubit0, qubit1);
                }
            }
        }
        adjoint ... {
            Rxx(-theta, qubit0, qubit1);
        }
    }

    /// # Summary
    /// Applies a rotation about the $y$-axis by a given angle.
    ///
    /// # Description
    /// \begin{align}
    ///     R_y(\theta) \mathrel{:=}
    ///     e^{-i \theta \sigma_y / 2} = 
    ///     \begin{bmatrix}
    ///         \cos \frac{\theta}{2} & -\sin \frac{\theta}{2}  \\\\
    ///         \sin \frac{\theta}{2} & \cos \frac{\theta}{2}
    ///     \end{bmatrix}.
    /// \end{align}
    ///
    /// # Input
    /// ## theta
    /// Angle about which the qubit is to be rotated.
    /// ## qubit
    /// Qubit to which the gate should be applied.
    ///
    /// # Remarks
    /// Equivalent to:
    /// ```qsharp
    /// R(PauliY, theta, qubit);
    /// ```
    operation Ry(theta : Double, qubit : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__ry__body(theta, qubit);
        }
        controlled (ctls, ...) {
            if Length(ctls) == 0 {
                __quantum__qis__ry__body(theta, qubit);
            }
            else {
                within {
                    MapPauli(qubit, PauliZ, PauliY);
                }
                apply {
                    Controlled Rz(ctls, (theta, qubit));
                }
            }
        }
        adjoint ... {
            Ry(-theta, qubit);
        }
    }

    /// # Summary
    /// Applies the two qubit Ising $YY$ rotation gate.
    ///
    /// # Description
    /// \begin{align}
    ///     R_yy(\theta) \mathrel{:=}
    ///     \begin{bmatrix}
    ///         \cos \theta & 0 & 0 & i\sin \theta  \\\\
    ///         0 & \cos \theta & -i\sin \theta & 0  \\\\
    ///         0 & -i\sin \theta & \cos \theta & 0  \\\\
    ///         i\sin \theta & 0 & 0 & \cos \theta
    ///     \end{bmatrix}.
    /// \end{align}
    ///
    /// # Input
    /// ## theta
    /// The angle about which the qubits are rotated.
    /// ## qubit0
    /// The first qubit input to the gate.
    /// ## qubit1
    /// The second qubit input to the gate.
    operation Ryy(theta : Double, qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__ryy__body(theta, qubit0, qubit1);
        }
        controlled (ctls, ...) {
            if Length(ctls) == 0 {
                __quantum__qis__ryy__body(theta, qubit0, qubit1);
            }
            elif Length(ctls) == 1 {
                CRyy(ctls[0], theta, qubit0, qubit1);
            }
            else {
                use aux = Qubit[Length(ctls) - 1];
                within {
                    CollectControls(ctls, aux, 0);
                    AdjustForSingleControl(ctls, aux);
                }
                apply {
                    CRyy(aux[Length(ctls) - 2], theta, qubit0, qubit1);
                }
            }
        }
        adjoint ... {
            Ryy(-theta, qubit0, qubit1);
        }
    }

    /// # Summary
    /// Applies a rotation about the $z$-axis by a given angle.
    ///
    /// # Description
    /// \begin{align}
    ///     R_z(\theta) \mathrel{:=}
    ///     e^{-i \theta \sigma_z / 2} = 
    ///     \begin{bmatrix}
    ///         e^{-i \theta / 2} & 0 \\\\
    ///         0 & e^{i \theta / 2}
    ///     \end{bmatrix}.
    /// \end{align}
    ///
    /// # Input
    /// ## theta
    /// Angle about which the qubit is to be rotated.
    /// ## qubit
    /// Qubit to which the gate should be applied.
    ///
    /// # Remarks
    /// Equivalent to:
    /// ```qsharp
    /// R(PauliZ, theta, qubit);
    /// ```
    operation Rz(theta : Double, qubit : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__rz__body(theta, qubit);
        }
        controlled (ctls, ...) {
            if Length(ctls) == 0 {
                __quantum__qis__rz__body(theta, qubit);
            }
            elif Length(ctls) == 1 {
                CRz(ctls[0], theta, qubit);
            }
            else {
                use aux = Qubit[Length(ctls) - 1];
                within {
                    CollectControls(ctls, aux, 0);
                    AdjustForSingleControl(ctls, aux);
                }
                apply {
                    CRz(aux[Length(ctls) - 2], theta, qubit);
                }
            }
        }
        adjoint ... {
            Rz(-theta, qubit);
        }
    }

    /// # Summary
    /// Applies the two qubit Ising $ZZ$ rotation gate.
    ///
    /// # Description
    /// \begin{align}
    ///     R_zz(\theta) \mathrel{:=}
    ///     \begin{bmatrix}
    ///         e^{-i \theta / 2} & 0 & 0 & 0 \\\\
    ///         0 & e^{i \theta / 2} & 0 & 0 \\\\
    ///         0 & 0 & e^{i \theta / 2} & 0 \\\\
    ///         0 & 0 & 0 & e^{-i \theta / 2}
    ///     \end{bmatrix}.
    /// \end{align}
    ///
    /// # Input
    /// ## theta
    /// The angle about which the qubits are rotated.
    /// ## qubit0
    /// The first qubit input to the gate.
    /// ## qubit1
    /// The second qubit input to the gate.
    operation Rzz(theta : Double, qubit0 : Qubit, qubit1 : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__rzz__body(theta, qubit0, qubit1);
        }
        controlled (ctls, ...) {
            if Length(ctls) == 0 {
                __quantum__qis__rzz__body(theta, qubit0, qubit1);
            }
            elif Length(ctls) == 1 {
                CRzz(ctls[0], theta, qubit0, qubit1);
            }
            else {
                use aux = Qubit[Length(ctls) - 1];
                within {
                    CollectControls(ctls, aux, 0);
                    AdjustForSingleControl(ctls, aux);
                }
                apply {
                    CRzz(aux[Length(ctls) - 2], theta, qubit0, qubit1);
                }
            }
        }
        adjoint ... {
            Rzz(-theta, qubit0, qubit1);
        }
    }

    /// # Summary
    /// Applies the π/4 phase gate to a single qubit.
    ///
    /// # Description
    /// \begin{align}
    ///     S \mathrel{:=}
    ///     \begin{bmatrix}
    ///         1 & 0 \\\\
    ///         0 & i
    ///     \end{bmatrix}.
    /// \end{align}
    ///
    /// # Input
    /// ## qubit
    /// Qubit to which the gate should be applied.
    operation S(qubit : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__s__body(qubit);
        }
        adjoint ... {
            __quantum__qis__s__adj(qubit);
        }
        controlled (ctls, ...) {
            if Length(ctls) == 0 {
                __quantum__qis__s__body(qubit);
            }
            elif Length(ctls) == 1 {
                CS(ctls[0], qubit);
            }
            elif Length(ctls) == 2 {
                Controlled CS([ctls[0]], (ctls[1], qubit));
            }
            else {
                use aux = Qubit[Length(ctls) - 2];
                within {
                    CollectControls(ctls, aux, 1 - (Length(ctls) % 2));
                }
                apply {
                    if Length(ctls) % 2 != 0 {
                        Controlled CS([ctls[Length(ctls) - 1]], (aux[Length(ctls) - 3], qubit));
                    }
                    else {
                        Controlled CS([aux[Length(ctls) - 3]], (aux[Length(ctls) - 4], qubit));
                    }
                }
            }
        }
        controlled adjoint (ctls, ...) {
            if Length(ctls) == 0 {
                __quantum__qis__s__adj(qubit);
            }
            elif Length(ctls) == 1 {
                Adjoint CS(ctls[0], qubit);
            }
            elif Length(ctls) == 2 {
                Controlled Adjoint CS([ctls[0]], (ctls[1], qubit));
            }
            else {
                use aux = Qubit[Length(ctls) - 2];
                within {
                    CollectControls(ctls, aux, 1 - (Length(ctls) % 2));
                }
                apply {
                    if Length(ctls) % 2 != 0 {
                        Controlled Adjoint CS([ctls[Length(ctls) - 1]], (aux[Length(ctls) - 3], qubit));
                    }
                    else {
                        Controlled Adjoint CS([aux[Length(ctls) - 3]], (aux[Length(ctls) - 4], qubit));
                    }
                }
            }
        }
    }

    /// # Summary
    /// Applies the SWAP gate to a pair of qubits.
    ///
    /// # Description
    /// \begin{align}
    ///     \operatorname{SWAP} \mathrel{:=}
    ///     \begin{bmatrix}
    ///         1 & 0 & 0 & 0 \\\\
    ///         0 & 0 & 1 & 0 \\\\
    ///         0 & 1 & 0 & 0 \\\\
    ///         0 & 0 & 0 & 1
    ///     \end{bmatrix},
    /// \end{align}
    ///
    /// where rows and columns are ordered as in the quantum concepts guide.
    ///
    /// # Input
    /// ## qubit1
    /// First qubit to be swapped.
    /// ## qubit2
    /// Second qubit to be swapped.
    ///
    /// # Remarks
    /// Equivalent to:
    /// ```qsharp
    /// CNOT(qubit1, qubit2);
    /// CNOT(qubit2, qubit1);
    /// CNOT(qubit1, qubit2);
    /// ```
    operation SWAP(qubit1 : Qubit, qubit2 : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__swap__body(qubit1, qubit2);
        }
        adjoint self;
        controlled (ctls, ...) {
            if (Length(ctls) == 0) {
                __quantum__qis__swap__body(qubit1, qubit2);
            }
            else {
                within {
                    CNOT(qubit1, qubit2);
                }
                apply {
                    Controlled CNOT(ctls, (qubit2, qubit1));
                }
            }
        }
    }

    /// # Summary
    /// Applies the π/8 gate to a single qubit.
    ///
    /// # Description
    /// \begin{align}
    ///     T \mathrel{:=}
    ///     \begin{bmatrix}
    ///         1 & 0 \\\\
    ///         0 & e^{i \pi / 4}
    ///     \end{bmatrix}.
    /// \end{align}
    ///
    /// # Input
    /// ## qubit
    /// Qubit to which the gate should be applied.
    operation T(qubit : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__t__body(qubit);
        }
        adjoint ... {
            __quantum__qis__t__adj(qubit);
        }
        controlled (ctls, ...) {
            if Length(ctls) == 0 {
                __quantum__qis__t__body(qubit);
            }
            elif Length(ctls) == 1 {
                CT(ctls[0], qubit);
            }
            else {
                use aux = Qubit[Length(ctls) - 1];
                within {
                    CollectControls(ctls, aux, 0);
                    AdjustForSingleControl(ctls, aux);
                }
                apply {
                    CT(aux[Length(ctls) - 2], qubit);
                }
            }
        }
        controlled adjoint (ctls, ...) {
            if Length(ctls) == 0 {
                __quantum__qis__t__adj(qubit);
            }
            elif Length(ctls) == 1 {
                Adjoint CT(ctls[0], qubit);
            }
            else {
                use aux = Qubit[Length(ctls) - 1];
                within {
                    CollectControls(ctls, aux, 0);
                    AdjustForSingleControl(ctls, aux);
                }
                apply {
                    Adjoint CT(aux[Length(ctls) - 2], qubit);
                }
            }
        }
    }

    /// # Summary
    /// Applies the Pauli $X$ gate.
    ///
    /// # Description
    /// \begin{align}
    ///     \sigma_x \mathrel{:=}
    ///     \begin{bmatrix}
    ///         0 & 1 \\\\
    ///         1 & 0
    ///     \end{bmatrix}.
    /// \end{align}
    ///
    /// # Input
    /// ## qubit
    /// Qubit to which the gate should be applied.
    operation X(qubit : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__x__body(qubit);
        }
        controlled (ctls, ...) {
            if Length(ctls) == 0 {
                __quantum__qis__x__body(qubit);
            }
            elif Length(ctls) == 1 {
                __quantum__qis__cx__body(ctls[0], qubit);
            }
            elif Length(ctls) == 2 {
                __quantum__qis__ccx__body(ctls[0], ctls[1], qubit);
            }
            else {
                use aux = Qubit[Length(ctls) - 2];
                within {
                    CollectControls(ctls, aux, 1 - (Length(ctls) % 2));
                }
                apply {
                    if Length(ctls) % 2 != 0 {
                        __quantum__qis__ccx__body(ctls[Length(ctls) - 1], aux[Length(ctls) - 3], qubit);
                    }
                    else {
                        __quantum__qis__ccx__body(aux[Length(ctls) - 3], aux[Length(ctls) - 4], qubit);
                    }
                }
            }
        }
        adjoint self;
    }

    /// # Summary
    /// Applies the Pauli $Y$ gate.
    ///
    /// # Description
    /// \begin{align}
    ///     \sigma_y \mathrel{:=}
    ///     \begin{bmatrix}
    ///         0 & -i \\\\
    ///         i & 0
    ///     \end{bmatrix}.
    /// \end{align}
    ///
    /// # Input
    /// ## qubit
    /// Qubit to which the gate should be applied.
    operation Y(qubit : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__y__body(qubit);
        }
        controlled (ctls, ...) {
            if (Length(ctls) == 0) {
                __quantum__qis__y__body(qubit);
            }
            elif (Length(ctls) == 1) {
                __quantum__qis__cy__body(ctls[0], qubit);
            }
            elif (Length(ctls) == 2) {
                CCY(ctls[0], ctls[1], qubit);
            }
            else {
                use aux = Qubit[Length(ctls) - 2];
                within {
                    CollectControls(ctls, aux, 1 - (Length(ctls) % 2));
                }
                apply {
                    if Length(ctls) % 2 != 0 {
                        CCY(ctls[Length(ctls) - 1], aux[Length(ctls) - 3], qubit);
                    }
                    else {
                        CCY(aux[Length(ctls) - 3], aux[Length(ctls) - 4], qubit);
                    }
                }
            }
        }
        adjoint self;
    }

    /// # Summary
    /// Applies the Pauli $Z$ gate.
    ///
    /// # Description
    /// \begin{align}
    ///     \sigma_z \mathrel{:=}
    ///     \begin{bmatrix}
    ///         1 & 0 \\\\
    ///         0 & -1
    ///     \end{bmatrix}.
    /// \end{align}
    ///
    /// # Input
    /// ## qubit
    /// Qubit to which the gate should be applied.
    operation Z(qubit : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__z__body(qubit);
        }
        controlled (ctls, ...) {
            if Length(ctls) == 0 {
                __quantum__qis__z__body(qubit);
            }
            elif Length(ctls) == 1 {
                __quantum__qis__cz__body(ctls[0], qubit);
            }
            elif Length(ctls) == 2 {
                CCZ(ctls[0], ctls[1], qubit);
            }
            else {
                use aux = Qubit[Length(ctls) - 2];
                within {
                    CollectControls(ctls, aux, 1 - (Length(ctls) % 2));
                }
                apply {
                    if Length(ctls) % 2 != 0 {
                        CCZ(ctls[Length(ctls) - 1], aux[Length(ctls) - 3], qubit);
                    }
                    else {
                        CCZ(aux[Length(ctls) - 3], aux[Length(ctls) - 4], qubit);
                    }
                }
            }
        }
        adjoint self;
    }

    /// # Summary
    /// Logs a message.
    ///
    /// # Input
    /// ## msg
    /// The message to be reported.
    ///
    /// # Remarks
    /// The specific behavior of this function is simulator-dependent,
    /// but in most cases the given message will be written to the console.
    /// ```
    function Message (msg : String) : Unit {
        body intrinsic;
    }


}
