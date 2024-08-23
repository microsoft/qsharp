// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Diagnostics {
    open QIR.Intrinsic;

    /// # Summary
    /// Dumps the current target machine's status.
    ///
    /// # Description
    /// This method allows you to dump information about the current quantum state.
    /// The actual information generated and the semantics are specific to each target machine.
    ///
    /// For the local sparse-state simulator distributed as part of the
    /// Quantum Development Kit, this method will write the wave function as a
    /// one-dimensional array of pairs of state indices and complex numbers, in which each element represents
    /// the amplitudes of the probability of measuring the corresponding state.
    ///
    /// # Example
    /// When run on the sparse-state simulator, the following snippet dumps
    /// the Bell state (|00⟩ + |11⟩ ) / √2 to the console:
    /// ```qsharp
    /// use left = Qubit();
    /// use right = Qubit();
    /// within {
    ///     H(left);
    ///     CNOT(left, right);
    /// } apply {
    ///     DumpMachine();
    /// }
    /// ```
    function DumpMachine() : Unit {
        body intrinsic;
    }

    /// # Summary
    /// Dumps the current target machine's status associated with the given qubits.
    ///
    /// # Input
    /// ## qubits
    /// The list of qubits to report.
    ///
    /// # Remarks
    /// This method allows you to dump the information associated with the state of the
    /// given qubits.
    ///
    /// For the local sparse-state simulator distributed as part of the
    /// Quantum Development Kit, this method will write the
    /// state of the given qubits (i.e. the wave function of the corresponding subsystem) as a
    /// one-dimensional array of pairs of state indices and complex numbers, in which each element represents
    /// the amplitudes of the probability of measuring the corresponding state.
    /// If the given qubits are entangled with some other qubit and their
    /// state can't be separated, it fails with a runtime error indicating that the qubits are entangled.
    ///
    /// # Example
    /// When run on the sparse-state simulator, the following snippet dumps
    /// the Bell state (|00⟩ + |11⟩ ) / √2 to the console:
    /// ```qsharp
    /// use left = Qubit();
    /// use right = Qubit();
    /// within {
    ///     H(left);
    ///     CNOT(left, right);
    /// } apply {
    ///     DumpRegister([left, right]);
    /// }
    /// ```
    function DumpRegister(register : Qubit[]) : Unit {
        body intrinsic;
    }

    /// # Summary
    /// Given an operation, dumps the matrix representation of the operation actiong on the given
    /// number of qubits.
    ///
    /// # Input
    /// ## nQubits
    /// The number of qubits on which the given operation acts.
    /// ## op
    /// The operation that is to be diagnosed.
    ///
    /// # Remarks
    /// When run on the sparse-state simulator, the following snippet
    /// will output the matrix
    /// $\left(\begin{matrix} 0.0 & 0.707 \\\\ 0.707 & 0.0\end{matrix}\right)$:
    ///
    /// ```qsharp
    /// operation DumpH() : Unit {
    ///     DumpOperation(1, qs => H(qs[0]));
    /// }
    /// ```
    /// Calling this operation has no observable effect from within Q#.
    /// Note that if `DumpOperation` is called when there are other qubits allocated,
    /// the matrix displayed may reflect any global phase that has accumulated on the other qubits.
    operation DumpOperation(nQubits : Int, op : Qubit[] => Unit) : Unit {
        use (targets, extra) = (Qubit[nQubits], Qubit[nQubits]);
        for i in 0..nQubits - 1 {
            H(targets[i]);
            CNOT(targets[i], extra[i]);
        }
        op(targets);
        DumpMatrix(targets + extra);
        ResetAll(targets + extra);
    }

    function DumpMatrix(qs : Qubit[]) : Unit {
        body intrinsic;
    }

    /// # Summary
    /// Checks whether a qubit is in the |0⟩ state, returning true if it is.
    ///
    /// # Description
    /// This operation checks whether a qubit is in the |0⟩ state. It will return true only
    /// if the qubit is deterministically in the |0⟩ state, and will return false otherwise. This operation
    /// does not change the state of the qubit.
    ///
    /// # Input
    /// ## qubit
    /// The qubit to check.
    /// # Output
    /// True if the qubit is in the |0⟩ state, false otherwise.
    ///
    /// # Remarks
    /// This operation is useful for checking whether a qubit is in the |0⟩ state during simulation. It is not possible to check
    /// this on hardware without measuring the qubit, which could change the state.
    @Config(Unrestricted)
    operation CheckZero(qubit : Qubit) : Bool {
        body intrinsic;
    }

    /// # Summary
    /// Checks whether all qubits in the provided array are in the |0⟩ state. Returns true if they are.
    ///
    /// # Description
    /// This operation checks whether all qubits in the provided array are in the |0⟩ state. It will return true only
    /// if all qubits are deterministically in the |0⟩ state, and will return false otherwise. This operation
    /// does not change the state of the qubits.
    ///
    /// # Input
    /// ## qubits
    /// The qubits to check.
    /// # Output
    /// True if all qubits are in the |0⟩ state, false otherwise.
    ///
    /// # Remarks
    /// This operation is useful for checking whether a qubit is in the |0⟩ state during simulation. It is not possible to check
    /// this on hardware without measuring the qubit, which could change the state.
    @Config(Unrestricted)
    operation CheckAllZero(qubits : Qubit[]) : Bool {
        for q in qubits {
            if not CheckZero(q) {
                return false;
            }
        }

        return true;
    }

    /// # Summary
    /// Checks whether a given condition is true, failing with a message if it is not.
    ///
    /// # Description
    /// This function checks whether a given condition is true. If the condition is false, the operation fails with the given message,
    /// terminating the program.
    ///
    /// # Input
    /// ## actual
    /// The condition to check.
    /// ## message
    /// The message to use in the failure if the condition is false.
    function Fact(actual : Bool, message : String) : Unit {
        if (not actual) {
            fail message;
        }
    }

    /// # Summary
    /// Given two operations, checks that they act identically for all input states.
    ///
    /// # Description
    /// This check is implemented by using the Choi–Jamiołkowski isomorphism to reduce
    /// this check to a check on two entangled registers.
    /// Thus, this operation needs only a single call to each operation being tested,
    /// but requires twice as many qubits to be allocated.
    /// This check can be used to ensure, for instance, that an optimized version of an
    /// operation acts identically to its naïve implementation, or that an operation
    /// which acts on a range of non-quantum inputs agrees with known cases.
    ///
    /// # Remarks
    /// This operation requires that the operation modeling the expected behavior is
    /// adjointable, so that the inverse can be performed on the target register alone.
    /// Formally, one can specify a transpose operation, which relaxes this requirement,
    /// but the transpose operation is not in general physically realizable for arbitrary
    /// quantum operations and thus is not included here as an option.
    ///
    /// # Input
    /// ## nQubits
    /// Number of qubits to pass to each operation.
    /// ## actual
    /// Operation to be tested.
    /// ## expected
    /// Operation defining the expected behavior for the operation under test.
    /// # Output
    /// True if operations are equal, false otherwise.
    @Config(Unrestricted)
    operation CheckOperationsAreEqual(
        nQubits : Int,
        actual : (Qubit[] => Unit),
        expected : (Qubit[] => Unit is Adj)
    ) : Bool {

        // Prepare a reference register entangled with the target register.
        use reference = Qubit[nQubits];
        use target = Qubit[nQubits];

        // Apply operations.
        within {
            for i in 0..nQubits - 1 {
                H(reference[i]);
                CNOT(reference[i], target[i]);
            }
        } apply {
            actual(target);
            Adjoint expected(target);
        }

        // Check and return result.
        let areEqual = CheckAllZero(reference) and CheckAllZero(target);
        ResetAll(target);
        ResetAll(reference);
        areEqual
    }

    export DumpMachine, DumpRegister, DumpOperation, CheckZero, CheckAllZero, Fact, CheckOperationsAreEqual;
}
