// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Canon {
    open QIR.Intrinsic;
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Arithmetic;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Arrays;

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

    /// # Summary
    /// Uses SWAP gates to Reversed the order of the qubits in
    /// a register.
    ///
    /// # Input
    /// ## register
    /// The qubits order of which should be reversed using SWAP gates
    operation SwapReverseRegister (register : Qubit[]) : Unit {
        body (...) {
            let totalQubits = Length(register);
            let halfTotal = totalQubits / 2;

            for i in 0 .. halfTotal - 1 {
                SWAP(register[i], register[(totalQubits - i) - 1]);
            }
        }

        adjoint self;
        // TODO: controlled distribute;
        // TODO: controlled adjoint self;
    }

    /// # Summary
    /// Apply the Approximate Quantum Fourier Transform (AQFT) to a quantum register.
    ///
    /// # Input
    /// ## a
    /// approximation parameter which determines at which level the controlled Z-rotations that
    /// occur in the QFT circuit are pruned.
    ///
    /// The approximation parameter a determines the pruning level of the Z-rotations, i.e.,
    /// a ∈ {0..n} and all Z-rotations 2π/2ᵏ where k>a are
    /// removed from the QFT circuit. It is known that for k >= log₂(n)+log₂(1/ε)+3
    /// one can bound ||QFT-AQFT||<ε.
    ///
    /// ## qs
    /// quantum register of n qubits to which the Approximate Quantum Fourier Transform is applied.
    ///
    /// # Remarks
    /// AQFT requires Z-rotation gates of the form 2π/2ᵏ and Hadamard gates.
    ///
    /// The input and output are assumed to be encoded in big endian encoding.
    ///
    ///
    /// # References
    /// - [ *M. Roetteler, Th. Beth*,
    ///      Appl. Algebra Eng. Commun. Comput.
    ///      19(3): 177-193 (2008) ](http://doi.org/10.1007/s00200-008-0072-2)
    /// - [ *D. Coppersmith* arXiv:quant-ph/0201067v1 ](https://arxiv.org/abs/quant-ph/0201067)
    operation ApproximateQFT (a : Int, qs : Qubit[]) : Unit {
        // TODO: is Adj + Ctl
        body (...) {
            let nQubits = Length(qs);
            Fact(nQubits > 0, "`Length(qs)` must be least 1");
            Fact(a > 0 and a <= nQubits, "`a` must be positive and less than `Length(qs)`");

            for i in 0 .. nQubits - 1 {
                for j in 0 .. i - 1 {
                    if i - j < a {
                        Controlled R1Frac([qs[i]], (1, i - j, (qs)[j]));
                    }
                }

                H(qs[i]);
            }

            // Apply the bit reversal permutation to the quantum register as
            // a side effect, such that we enforce the invariants specified
            // by the BigEndian UDT.
            SwapReverseRegister(qs);
        }

        adjoint (...) {
            // TODO: adjoint auto
            let nQubits = Length(qs);
            Fact(nQubits > 0, "`Length(qs)` must be least 1");
            Fact(a > 0 and a <= nQubits, "`a` must be positive and less than `Length(qs)`");

            Adjoint SwapReverseRegister(qs);


            for i in nQubits - 1 .. -1 .. 0  {
                Adjoint H(qs[i]);
                for j in i - 1 .. -1 .. 0 {
                    if i - j < a {
                        Controlled Adjoint R1Frac([qs[i]], (1, i - j, (qs)[j]));
                    }
                }

            }

        }
    }

    /// # Summary
    /// Performs the Quantum Fourier Transform on a quantum register containing an
    /// integer in the big-endian representation.
    ///
    /// # Input
    /// ## qs
    /// Quantum register to which the Quantum Fourier Transform is applied
    ///
    /// # Remarks
    /// The input and output are assumed to be in big endian encoding.
    ///
    /// # See Also
    /// - Microsoft.Quantum.Canon.ApproximateQFT
    /// - Microsoft.Quantum.Canon.QFTLE
    internal operation ApplyQuantumFourierTransformBE(qs : Qubit[]) : Unit {
        // TODO: is Adj + Ctl
        body (...) {
            ApproximateQFT(Length(qs), qs);
        }
        adjoint (...) {
            // TODO: adjoint auto
            Adjoint ApproximateQFT(Length(qs), qs);
        }
    }

    /// # Summary
    /// Performs the Quantum Fourier Transform on a quantum register containing an
    /// integer in the little-endian representation.
    ///
    /// # Input
    /// ## qs
    /// Quantum register to which the Quantum Fourier Transform is applied
    ///
    /// # Remarks
    /// The input and output are assumed to be in little endian encoding.
    ///
    ///
    /// # See Also
    /// - Microsoft.Quantum.Canon.ApplyQuantumFourierTransformBE
    operation ApplyQuantumFourierTransform(qs : Qubit[]) : Unit {
        // TODO: is Adj + Ctl
        ApplyQuantumFourierTransformBE(LittleEndianAsBigEndian(qs));
    }

    /// # Summary
    /// Performs the Quantum Fourier Transform on a quantum register containing an
    /// integer in the big-endian representation.
    ///
    /// # Input
    /// ## qs
    /// Quantum register to which the Quantum Fourier Transform is applied
    ///
    /// # Remarks
    /// The input and output are assumed to be in big endian encoding.
    ///
    /// # See Also
    /// - Microsoft.Quantum.Canon.ApplyQuantumFourierTransformBE
    operation QFT(qs : Qubit[]) : Unit {
        body (...) {
            ApplyQuantumFourierTransformBE(qs);
        }
        adjoint(...) {
            // TODO: adjoint invert
            Adjoint ApplyQuantumFourierTransformBE(qs);
        }
        // TODO: controlled distribute;
        // TODO: controlled adjoint distribute;
    }

    /// # Summary
    /// Performs the Quantum Fourier Transform on a quantum register containing an
    /// integer in the little-endian representation.
    ///
    /// # Input
    /// ## qs
    /// Quantum register to which the Quantum Fourier Transform is applied
    ///
    /// # Remarks
    /// The input and output are assumed to be in little endian encoding.
    ///
    ///
    /// # See Also
    /// - Microsoft.Quantum.Canon.QFT
    operation QFTLE(qs : Qubit[]) : Unit {
        body (...) {
            ApplyQuantumFourierTransform(qs);
        }

        // TODO: adjoint invert;
        // TODO: controlled distribute;
        // TODO: controlled adjoint distribute;
    }

    /// # Summary
    /// Applies a single-qubit operation to each element in a register.
    /// The modifier `CA` indicates that the single-qubit operation is controllable
    /// and adjointable.
    ///
    /// # Input
    /// ## singleElementOperation
    /// Operation to apply to each qubit.
    /// ## register
    /// Array of qubits on which to apply the given operation.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The target on which the operation acts.
    ///
    /// # Example
    /// Prepare a three-qubit $\ket{+}$ state:
    /// ```qsharp
    /// using (register = Qubit[3]) {
    ///     ApplyToEachCA(H, register);
    /// }
    /// ```
    ///
    /// # See Also
    /// - Microsoft.Quantum.Canon.ApplyToEach
    operation ApplyToEachCA<'T> (singleElementOperation : ('T => Unit is Adj + Ctl), register : 'T[])
    : Unit is Adj + Ctl {
        for idxQubit in IndexRange(register) {
            singleElementOperation(register[idxQubit]);
        }
    }


    /// # Summary
    /// Applies a single-qubit operation to each element in a register.
    /// The modifier `A` indicates that the single-qubit operation is adjointable.
    ///
    /// # Input
    /// ## singleElementOperation
    /// Operation to apply to each qubit.
    /// ## register
    /// Array of qubits on which to apply the given operation.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The target on which the operation acts.
    ///
    /// # Example
    /// Prepare a three-qubit $\ket{+}$ state:
    /// ```qsharp
    /// using (register = Qubit[3]) {
    ///     ApplyToEachA(H, register);
    /// }
    /// ```
    ///
    /// # See Also
    /// - Microsoft.Quantum.Canon.ApplyToEach
    operation ApplyToEachA<'T> (singleElementOperation : ('T => Unit is Adj), register : 'T[])
    : Unit is Adj {
        for idxQubit in IndexRange(register) {
            singleElementOperation(register[idxQubit]);
        }
    }


    /// # Summary
    /// Applies a single-qubit operation to each element in a register.
    /// The modifier `C` indicates that the single-qubit operation is controllable.
    ///
    /// # Input
    /// ## singleElementOperation
    /// Operation to apply to each qubit.
    /// ## register
    /// Array of qubits on which to apply the given operation.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The target on which the operation acts.
    ///
    /// # Example
    /// Prepare a three-qubit $\ket{+}$ state:
    /// ```qsharp
    /// using (register = Qubit[3]) {
    ///     ApplyToEachC(H, register);
    /// }
    /// ```
    ///
    /// # See Also
    /// - Microsoft.Quantum.Canon.ApplyToEach
    operation ApplyToEachC<'T> (singleElementOperation : ('T => Unit is Ctl), register : 'T[])
    : Unit is Ctl {
        body (...) {
            for idxQubit in IndexRange(register) {
                singleElementOperation(register[idxQubit]);
            }
        }
        controlled (ctls, ...) {
            // TODO: Auto
            for idxQubit in IndexRange(register) {
                Controlled singleElementOperation(ctls, register[idxQubit]);
            }
        }
    }


    /// # Summary
    /// Applies a single-qubit operation to each element in a register.
    ///
    /// # Input
    /// ## singleElementOperation
    /// Operation to apply to each qubit.
    /// ## register
    /// Array of qubits on which to apply the given operation.
    ///
    /// # Type Parameters
    /// ## 'T
    /// The target on which the operation acts.
    ///
    /// # Example
    /// Prepare a three-qubit $\ket{+}$ state:
    /// ```qsharp
    /// using (register = Qubit[3]) {
    ///     ApplyToEach(H, register);
    /// }
    /// ```
    ///
    /// # See Also
    /// - Microsoft.Quantum.Canon.ApplyToEachC
    /// - Microsoft.Quantum.Canon.ApplyToEachA
    /// - Microsoft.Quantum.Canon.ApplyToEachCA
    operation ApplyToEach<'T> (singleElementOperation : ('T => Unit), register : 'T[]) : Unit {
        for idxQubit in IndexRange(register) {
            singleElementOperation(register[idxQubit]);
        }
    }


}
