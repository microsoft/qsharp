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
    /// the Bell state (|00〉 + |11〉 ) / √2 to the console:
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
    /// the Bell state (|00〉 + |11〉 ) / √2 to the console:
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

    @Config(Unrestricted)
    operation CheckZero(qubit : Qubit) : Bool {
        body intrinsic;
    }

    @Config(Unrestricted)
    operation CheckAllZero(qubits : Qubit[]) : Bool {
        for q in qubits {
            if not CheckZero(q) {
                return false;
            }
        }

        return true;
    }

    /// Checks whether a classical condition is true, and throws an exception if it is not.
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

    export DumpMachine, DumpRegister, CheckZero, CheckAllZero, Fact, CheckOperationsAreEqual;
}
