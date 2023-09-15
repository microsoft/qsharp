// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Katas {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;

    /// # Summary
    /// Given two operations, checks whether they act identically for all input states.
    /// This operation is implemented by using the Choi–Jamiołkowski isomorphism.
    ///
    /// N.B. Investigate whether this can be done using just 1 control qubit (GitHub issue #535).
    operation CheckOperationsEquivalence(
        op : (Qubit[] => Unit is Adj + Ctl),
        reference : (Qubit[] => Unit is Adj + Ctl),
        inputSize : Int)
    : Bool {
        Fact(inputSize > 0, "`inputSize` must be positive");
        use (control, target) = (Qubit[inputSize], Qubit[inputSize]);
        within {
            // N.B. The order in which quantum registers are passed to this operation is important.
            EntangleRegisters(control, target);
        }
        apply {
            op(target);
            Adjoint reference(target);
        }

        let areEquivalent = CheckAllZero(control + target);
        ResetAll(control + target);
        areEquivalent
    }

    /// # Summary
    /// Given two operations, checks whether they act identically (including global phase) for all input states.
    /// This is done through controlled versions of the operations instead of plain ones which convert the global phase
    /// into a relative phase that can be detected.
    operation CheckOperationsEquivalenceStrict(
        op : (Qubit[] => Unit is Adj + Ctl),
        reference : (Qubit[] => Unit is Adj + Ctl),
        inputSize : Int)
    : Bool {
        Fact(inputSize > 0, "`inputSize` must be positive");
        let controlledOp = register => Controlled op(register[...0], register[1...]);
        let controlledReference = register => Controlled reference(register[...0], register[1...]);
        let areEquivalent = CheckOperationsEquivalence(controlledOp, controlledReference, inputSize + 1);
        areEquivalent
    }

    /// # Summary
    /// Given two operations, checks whether they act identically on the zero state |0〉 ⊗ |0〉 ⊗ ... ⊗ |0〉 composed of
    /// `inputSize` qubits.
    operation CheckOperationsEquivalenceOnZeroState(
        op : (Qubit[] => Unit is Adj + Ctl),
        reference : (Qubit[] => Unit is Adj + Ctl),
        inputSize : Int)
    : Bool {
        Fact(inputSize > 0, "`inputSize` must be positive");
        use target = Qubit[inputSize];
        op(target);
        Adjoint reference(target);
        let isCorrect = CheckAllZero(target);
        ResetAll(target);
        isCorrect
    }

    /// # Summary
    /// Given two operations, checks whether they act identically on the zero state |0〉 ⊗ |0〉 ⊗ ... ⊗ |0〉 composed of
    /// `inputSize` qubits.
    /// This operation introduces a control qubit to convert a global phase into a relative phase to be able to detect
    /// it.
    operation CheckOperationsEquivalenceOnZeroStateStrict(
        op : (Qubit[] => Unit is Adj + Ctl),
        reference : (Qubit[] => Unit is Adj + Ctl),
        inputSize : Int)
    : Bool {
        Fact(inputSize > 0, "`inputSize` must be positive");
        use control = Qubit();
        use target = Qubit[inputSize];
        within {
            H(control);
        }
        apply {
            Controlled op([control], target);
            Adjoint Controlled reference([control], target);
        }

        let isCorrect = CheckAllZero([control] + target);
        ResetAll([control] + target);
        isCorrect
    }

    /// # Summary
    /// Shows the effect a quantum operation has on the quantum state.
    operation ShowEffectOnQuantumState(targetRegister : Qubit[], op : (Qubit[] => Unit is Adj + Ctl)) : Unit {
        Message("Quantum state before applying the operation:");
        DumpMachine();

        // Apply the operation, dump the simulator state and "undo" the operation by applying the adjoint.
        Message("Quantum state after applying the operation:");
        op(targetRegister);
        DumpMachine();
        Adjoint op(targetRegister);
    }

    /// # Summary
    /// Shows the comparison of the quantum state between a specific operation and a reference operation.
    operation ShowQuantumStateComparison(
        targetRegister : Qubit[],
        op : (Qubit[] => Unit is Adj + Ctl),
        reference : (Qubit[] => Unit is Adj + Ctl))
    : Unit {
        Message("Initial quantum state:");
        DumpMachine();

        // Apply the reference operation, dump the simulator state and "undo" the operation by applying the adjoint.
        reference(targetRegister);
        Message("Expected quantum state after applying the operation:");
        DumpMachine();
        Adjoint reference(targetRegister);

        // Apply the specific operation, dump the simulator state and "undo" the operation by applying the adjoint.
        op(targetRegister);
        Message("Actual quantum state after applying the operation:");
        DumpMachine();
        Adjoint op(targetRegister);
    }

    internal operation EntangleRegisters(
        control : Qubit[],
        target : Qubit[]) : Unit is Adj + Ctl {
        Fact(
            Length(control) == Length(target),
            $"The length of qubit registers must be the same.");

        for index in IndexRange(control) {
            H(control[index]);
            CNOT(control[index], target[index]);
        }
    }
}
