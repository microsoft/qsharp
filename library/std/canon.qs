// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Canon {
    open QIR.Intrinsic;
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;

    /// # Summary
    /// Applies an operation to each element in a register.
    ///
    /// # Input
    /// ## singleElementOperation
    /// Operation to apply to each element.
    /// ## register
    /// Array of elements on which to apply the given operation.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The target on which the operation acts.
    ///
    /// # Example
    /// Prepare a three-qubit $\ket{+}$ state:
    /// ```qsharp
    /// use register = Qubit[3];
    /// ApplyToEach(H, register);
    /// ```
    operation ApplyToEach<'T> (singleElementOperation : ('T => Unit), register : 'T[]) : Unit {
        for item in register {
            singleElementOperation(item);
        }
    }

    /// # Summary
    /// Applies an operation to each element in a register.
    /// The modifier `A` indicates that the single-element operation is adjointable.
    ///
    /// # Input
    /// ## singleElementOperation
    /// Operation to apply to each element.
    /// ## register
    /// Array of elements on which to apply the given operation.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The target on which the operation acts.
    ///
    /// # Example
    /// Prepare a three-qubit $\ket{+}$ state:
    /// ```qsharp
    /// use register = Qubit[3];
    /// ApplyToEach(H, register);
    /// ```
    ///
    /// # See Also
    /// - Microsoft.Quantum.Canon.ApplyToEach
    operation ApplyToEachA<'T> (singleElementOperation : ('T => Unit is Adj), register : 'T[])
    : Unit is Adj {
        for item in register {
            singleElementOperation(item);
        }
    }

    /// # Summary
    /// Applies an operation to each element in a register.
    /// The modifier `C` indicates that the single-element operation is controllable.
    ///
    /// # Input
    /// ## singleElementOperation
    /// Operation to apply to each element.
    /// ## register
    /// Array of elements on which to apply the given operation.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The target on which the operation acts.
    ///
    /// # Example
    /// Prepare a three-qubit $\ket{+}$ state:
    /// ```qsharp
    /// use register = Qubit[3];
    /// ApplyToEach(H, register);
    /// ```
    ///
    /// # See Also
    /// - Microsoft.Quantum.Canon.ApplyToEach
    operation ApplyToEachC<'T> (singleElementOperation : ('T => Unit is Ctl), register : 'T[])
    : Unit is Ctl {
        for item in register {
            singleElementOperation(item);
        }
    }

    /// # Summary
    /// Applies an operation to each element in a register.
    /// The modifier `CA` indicates that the single-element operation is controllable and adjointable.
    ///
    /// # Input
    /// ## singleElementOperation
    /// Operation to apply to each element.
    /// ## register
    /// Array of elements on which to apply the given operation.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The target on which the operation acts.
    ///
    /// # Example
    /// Prepare a three-qubit $\ket{+}$ state:
    /// ```qsharp
    /// use register = Qubit[3];
    /// ApplyToEach(H, register);
    /// ```
    ///
    /// # See Also
    /// - Microsoft.Quantum.Canon.ApplyToEach
    operation ApplyToEachCA<'T> (singleElementOperation : ('T => Unit is Adj + Ctl), register : 'T[])
    : Unit is Adj + Ctl {
        for item in register {
            singleElementOperation(item);
        }
    }

    /// # Summary
    /// Applies the controlled-X (CX) gate to a pair of qubits.
    ///
    /// # Description
    /// This operation can be simulated by the unitary matrix
    /// $$
    /// \begin{align}
    ///     \left(\begin{matrix}
    ///         1 & 0 & 0 & 0 \\\\
    ///         0 & 1 & 0 & 0 \\\\
    ///         0 & 0 & 0 & 1 \\\\
    ///         0 & 0 & 1 & 0
    ///      \end{matrix}\right)
    /// \end{align},
    /// $$
    /// where rows and columns are organized as in the quantum concepts guide.
    ///
    /// # Input
    /// ## control
    /// Control qubit for the CX gate.
    /// ## target
    /// Target qubit for the CX gate.
    ///
    /// # Remarks
    /// Equivalent to:
    /// ```qsharp
    /// Controlled X([control], target);
    /// ```
    /// and to:
    /// ```qsharp
    /// CNOT(control, target);
    /// ```
    operation CX(control : Qubit, target : Qubit) : Unit is Adj + Ctl{
        body ... {
            __quantum__qis__cx__body(control, target);
        }
        controlled (ctls, ...) {
            Controlled X(ctls + [control], target);
        }
        adjoint self;
    }

    /// # Summary
    /// Applies the controlled-Y (CY) gate to a pair of qubits.
    ///
    /// # Description
    /// This operation can be simulated by the unitary matrix
    /// $$
    /// \begin{align}
    ///     1 & 0 & 0 & 0 \\\\
    ///     0 & 1 & 0 & 0 \\\\
    ///     0 & 0 & 0 & -i \\\\
    ///     0 & 0 & i & 0
    /// \end{align},
    /// $$
    /// where rows and columns are organized as in the quantum concepts guide.
    ///
    /// # Input
    /// ## control
    /// Control qubit for the CY gate.
    /// ## target
    /// Target qubit for the CY gate.
    ///
    /// # Remarks
    /// Equivalent to:
    /// ```qsharp
    /// Controlled Y([control], target);
    /// ```
    operation CY(control : Qubit, target : Qubit) : Unit is Adj + Ctl{
        body ... {
            __quantum__qis__cy__body(control, target);
        }
        controlled (ctls, ...) {
            Controlled Y(ctls + [control], target);
        }
        adjoint self;
    }

    /// # Summary
    /// Applies the controlled-Z (CZ) gate to a pair of qubits.
    ///
    /// # Description
    /// This operation can be simulated by the unitary matrix
    /// $$
    /// \begin{align}
    ///     1 & 0 & 0 & 0 \\\\
    ///     0 & 1 & 0 & 0 \\\\
    ///     0 & 0 & 1 & 0 \\\\
    ///     0 & 0 & 0 & -1
    /// \end{align},
    /// $$
    /// where rows and columns are organized as in the quantum concepts guide.
    ///
    /// # Input
    /// ## control
    /// Control qubit for the CZ gate.
    /// ## target
    /// Target qubit for the CZ gate.
    ///
    /// # Remarks
    /// Equivalent to:
    /// ```qsharp
    /// Controlled Z([control], target);
    /// ```
    operation CZ(control : Qubit, target : Qubit) : Unit is Adj + Ctl {
        body ... {
            __quantum__qis__cz__body(control, target);
        }
        controlled (ctls, ...) {
            Controlled Z(ctls + [control], target);
        }
        adjoint self;
    }

    /// Given a pair, returns its first element.
    function Fst<'T, 'U> (pair : ('T, 'U)) : 'T {
        let (fst, _) = pair;
        return fst;
    }

    /// Given a pair, returns its second element.
    function Snd<'T, 'U> (pair : ('T, 'U)) : 'U {
        let (_, snd) = pair;
        return snd;
    }

    /// # Summary
    /// Computes the parity of a register of qubits in-place.
    ///
    /// # Description
    /// This operation transforms the state of its input as
    /// $$
    /// \begin{align}
    ///     \ket{q_0} \ket{q_1} \cdots \ket{q_{n - 1}} & \mapsto
    ///     \ket{q_0} \ket{q_0 \oplus q_1} \ket{q_0 \oplus q_1 \oplus q_2} \cdots
    ///         \ket{q_0 \oplus \cdots \oplus q_{n - 1}}.
    /// \end{align}
    /// $$
    ///
    /// # Input
    /// ## qubits
    /// Array of qubits whose parity is to be computed and stored.
    operation ApplyCNOTChain(qubits : Qubit[]) : Unit is Adj + Ctl {
        for i in 0..Length(qubits)-2 {
            CNOT(qubits[i], qubits[i+1]);
        }
    }

    /// # Summary
    /// Given a single-qubit Pauli operator, applies the corresponding operation
    /// to a single qubit.
    ///
    /// # Input
    /// ## pauli
    /// The Pauli operator to be applied.
    /// ## target
    /// The qubit to which `pauli` is to be applied as an operation.
    ///
    /// # Example
    /// The following are equivalent:
    /// ```qsharp
    /// ApplyP(PauliX, q);
    /// ```
    /// and
    /// ```qsharp
    /// X(q);
    /// ```
    operation ApplyP(pauli: Pauli, target : Qubit): Unit is Adj + Ctl {
        if   pauli == PauliX { X(target); }
        elif pauli == PauliY { Y(target); }
        elif pauli == PauliZ { Z(target); }
    }

    /// # Summary
    /// Given a multi-qubit Pauli operator, applies the corresponding operation
    /// to a quantum register.
    ///
    /// # Input
    /// ## pauli
    /// A multi-qubit Pauli operator represented as an array of single-qubit Pauli operators.
    /// ## target
    /// Register to apply the given Pauli operation on.
    ///
    /// # Example
    /// The following are equivalent:
    /// ```qsharp
    /// ApplyPauli([PauliY, PauliZ, PauliX], target);
    /// ```
    /// and
    /// ```qsharp
    /// Y(target[0]);
    /// Z(target[1]);
    /// X(target[2]);
    /// ```
    operation ApplyPauli(pauli: Pauli[], target: Qubit[]) : Unit is Adj + Ctl {
        Fact(Length(pauli) == Length(target), "`pauli` and `target` must be of the same length.");
        for i in 0..Length(pauli)-1 {
            ApplyP(pauli[i], target[i]);
        }
    }

    /// # Summary
    /// Applies a Pauli operator on each qubit in an array if the corresponding
    /// bit of a Boolean array matches a given input.
    ///
    /// # Input
    /// ## pauli
    /// Pauli operator to apply to `qubits[idx]` where `bitApply == bits[idx]`
    /// ## bitApply
    /// apply Pauli if bit is this value
    /// ## bits
    /// Boolean register specifying which corresponding qubit in `qubits` should be operated on
    /// ## qubits
    /// Quantum register on which to selectively apply the specified Pauli operator
    ///
    /// # Remarks
    /// The Boolean array and the quantum register must be of equal length.
    ///
    /// # Example
    /// The following applies an X operation on qubits 0 and 2, and a Z operation on qubits 1 and 3.
    /// ```qsharp
    /// use qubits = Qubit[4];
    /// let bits = [true, false, true, false];
    /// // Apply when index in `bits` is `true`.
    /// ApplyPauliFromBitString(PauliX, true, bits, qubits);
    /// // Apply when index in `bits` is `false`.
    /// ApplyPauliFromBitString(PauliZ, false, bits, qubits);
    /// ```
    operation ApplyPauliFromBitString(pauli : Pauli, bitApply : Bool, bits : Bool[], qubits : Qubit[])
    : Unit is Adj + Ctl {
        let nBits = Length(bits);
        Fact(nBits == Length(qubits), "Number of control bits must be equal to number of control qubits.");
        for i in 0..nBits-1 {
            if bits[i] == bitApply {
                ApplyP(pauli, qubits[i]);
            }
        }
    }

    /// # Summary
    /// Applies a Pauli operator on each qubit in an array if the corresponding
    /// bit of a Little-endian integer matches a given input.
    ///
    /// # Input
    /// ## pauli
    /// Pauli operator to apply to `qubits[idx]` when bit of numberState
    /// in idx position is the same as bitApply.
    /// ## bitApply
    /// apply Pauli if bit is this value
    /// ## numberState
    /// Little-endian integer specifying which corresponding qubit in `qubits` should be operated on
    /// ## qubits
    /// Quantum register on which to selectively apply the specified Pauli operator
    ///
    /// # Example
    /// The following applies an X operation on qubits 0 and 2, and a Z operation on qubits 1 and 3.
    /// ```qsharp
    /// use qubits = Qubit[4];
    /// let n = 5;
    /// // Apply when index in `bits` is `true`.
    /// ApplyPauliFromBitString(PauliX, true, n, qubits);
    /// // Apply when index in `bits` is `false`.
    /// ApplyPauliFromBitString(PauliZ, false, n, qubits);
    /// ```
    operation ApplyPauliFromInt(
        pauli: Pauli,
        bitApply: Bool,
        numberState: Int,
        qubits: Qubit[]): Unit is Adj + Ctl {

        let length = Length(qubits);
        Fact(numberState >= 0, "number must be non-negative");
        Fact(BitSizeI(numberState) <= length, "Bit size of numberState must not exceed qubits length");

        for i in 0..length-1 {
            // If we assume loop unrolling, 2^i will be optimized to a constant.
            if ((numberState &&& (1 <<< i)) != 0) == bitApply {
                ApplyP(pauli, qubits[i]);
            }
        }
    }

    /// # Summary
    /// Applies a unitary operation on the target if the control
    /// register state corresponds to a specified nonnegative integer.
    ///
    /// # Input
    /// ## numberState
    /// A nonnegative integer on which the operation `oracle` should be
    /// controlled.
    /// ## oracle
    /// A unitary operation to be controlled.
    /// ## target
    /// A target on which to apply `oracle`.
    /// ## controlRegister
    /// A quantum register that controls application of `oracle`.
    ///
    /// # Remarks
    /// The value of `numberState` is interpreted using a little-endian encoding.
    ///
    /// `numberState` must be at most $2^\texttt{Length(controlRegister)} - 1$.
    /// For example, `numberState = 537` means that `oracle`
    /// is applied if and only if `controlRegister` is in the state $\ket{537}$.
    operation ApplyControlledOnInt<'T>(
        numberState: Int,
        oracle: ('T => Unit is Adj + Ctl),
        controlRegister: Qubit[],
        target: 'T): Unit is Adj + Ctl {

        within {
            ApplyPauliFromInt(PauliX, false, numberState, controlRegister);
        } apply {
            Controlled oracle(controlRegister, target);
        }
    }

    /// # Summary
    /// Applies a unitary operation on the target,
    /// controlled on a state specified by a given bit mask.
    ///
    /// # Input
    /// ## bits
    /// The bit string to control the given unitary operation on.
    /// ## oracle
    /// The unitary operation to be applied on the target.
    /// ## target
    /// The target to be passed to `oracle` as an input.
    /// ## controlRegister
    /// A quantum register that controls application of `oracle`.
    ///
    /// # Remarks
    /// The pattern given by `bits` may be shorter than `controlRegister`,
    /// in which case additional control qubits are ignored (that is, neither
    /// controlled on $\ket{0}$ nor $\ket{1}$).
    /// If `bits` is longer than `controlRegister`, an error is raised.
    ///
    /// For example, `bits = [0,1,0,0,1]` means that `oracle` is applied if and only if `controlRegister`
    /// is in the state $\ket{0}\ket{1}\ket{0}\ket{0}\ket{1}$.
    operation ApplyControlledOnBitString<'T>(
        bits: Bool[],
        oracle: ('T => Unit is Adj + Ctl),
        controlRegister: Qubit[],
        target: 'T): Unit is Adj + Ctl {

        // The control register must have enough bits to implement the requested control.
        Fact(Length(bits) <= Length(controlRegister), "Control register shorter than control pattern.");

        // Use a subregister of the controlled register when
        // bits is shorter than controlRegister.
        let controlSubregister = controlRegister[...Length(bits) - 1];
        within {
            ApplyPauliFromBitString(PauliX, false, bits, controlSubregister);
        } apply {
            Controlled oracle(controlSubregister, target);
        }
    }

}
