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

    /// # Summary
    /// Starts counting the number of times the given operation is called. Fails if the operation is already being counted.
    ///
    /// # Description
    /// This operation allows you to count the number of times a given operation is called. As part of
    /// starting the counting, the counter is reset to zero, which may override existing counts for the same operation.
    /// Counting is based on the specific specialization of the operation invoked, so `X` and `Adjoint X` are counted separately.
    /// Likewise `Controlled X`, `CNOT`, and `CX` are independent operations that are counted separately, as are `Controlled X`
    /// and `Controlled Adjoint X`.
    ///
    /// # Input
    /// ## callable
    /// The operation to be counted.
    ///
    /// # Remarks
    /// Counting operation calls requires specific care in what operation is passed as input. For example, `StartCountingOperation(H)` will
    /// count only the number of times `H` is called, while `StartCountingOperation(Adjoint H)` will count only the number of times `Adjoint H` is called, even
    /// though `H` is self-adjoint. This is due to how the execution treats the invocation of these operations as distinct by their specialization.
    /// In the same way, `StartCountingOperation(Controlled X)` will count only the number of times `Controlled X` is called, while
    /// `StartCountingOperation(CNOT)` will count only the number of times `CNOT` is called.
    ///
    /// When counting lambdas, the symbol the lambda is bound to is used to identify the operation and it is counted as a separate operation. For example,
    /// ```qsharp
    /// let myOp = q => H(q);
    /// StartCountingOperation(myOp);
    /// ```
    /// Will count specifically calls to `myOp` and not `H`. By contrast, the following code will count calls to `H` itself:
    /// ```qsharp
    /// let myOp = H;
    /// StartCountingOperation(myOp);
    /// ```
    /// This is because this code does not define a lambda and instead just creates a binding to `H` directly.
    @Config(Unrestricted)
    operation StartCountingOperation<'In, 'Out>(callable : 'In => 'Out) : Unit {
        body intrinsic;
    }

    /// # Summary
    /// Stops counting the number of times the given operation is called and returns the count. Fails
    /// if the operation was not being counted.
    ///
    /// # Description
    /// This operation allows you to stop counting the number of times a given operation is called and returns the count.
    /// If the operation was not being counted, it triggers a runtime failure.
    ///
    /// # Input
    /// ## callable
    /// The operation whose count will be returned.
    /// # Output
    /// The number of times the operation was called since the last call to `StartCountingOperation`.
    @Config(Unrestricted)
    operation StopCountingOperation<'In, 'Out>(callable : 'In => 'Out) : Int {
        body intrinsic;
    }

    /// # Summary
    /// Starts counting the number of times the given function is called. Fails if the function is already being counted.
    ///
    /// # Description
    /// This operation allows you to count the number of times a given function is called. As part of
    /// starting the counting, the counter is reset to zero, which may override existing counts for the same function.
    ///
    /// # Input
    /// ## callable
    /// The function to be counted.
    ///
    /// # Remarks
    /// When counting lambdas, the symbol the lambda is bound to is used to identify the function and it is counted as a separate function. For example,
    /// ```qsharp
    /// let myFunc = i -> AbsI(i);
    /// StartCountingFunction(myFunc);
    /// ```
    /// Will count specifically calls to `myFunc` and not `AbsI`. By contrast, the following code will count calls to `AbsI` itself:
    /// ```qsharp
    /// let myFunc = AbsI;
    /// StartCountingFunction(myFunc);
    /// ```
    /// This is because this code does not define a lambda and instead just creates a binding to `AbsI` directly.
    @Config(Unrestricted)
    operation StartCountingFunction<'In, 'Out>(callable : 'In -> 'Out) : Unit {
        body intrinsic;
    }

    /// # Summary
    /// Stops counting the number of times the given function is called and returns the count. Fails
    /// if the function was not being counted.
    ///
    /// # Description
    /// This operation allows you to stop counting the number of times a given function is called and returns the count.
    /// If the function was not being counted, it triggers a runtime failure.
    ///
    /// # Input
    /// ## callable
    /// The function whose count will be returned.
    /// # Output
    /// The number of times the function was called since the last call to `StartCountingFunction`.
    @Config(Unrestricted)
    operation StopCountingFunction<'In, 'Out>(callable : 'In -> 'Out) : Int {
        body intrinsic;
    }

    export DumpMachine, DumpRegister, CheckZero, CheckAllZero, Fact, CheckOperationsAreEqual, StartCountingOperation, StopCountingOperation, StartCountingFunction, StopCountingFunction;
}
