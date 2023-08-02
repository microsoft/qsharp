// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Katas {
    open Microsoft.Quantum.Arrays;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;

    operation CheckOperationsEquivalence(
        op : (Qubit[] => Unit is Adj + Ctl),
        reference : (Qubit[] => Unit is Adj + Ctl),
        inputSize: Int)
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

    operation CheckOperationsEquivalenceStrict(
        op : (Qubit[] => Unit is Adj + Ctl),
        reference : (Qubit[] => Unit is Adj + Ctl),
        inputSize: Int)
    : Bool {
        Fact(inputSize > 0, "`inputSize` must be positive");
        let controlledOp = register => Controlled op(register[...0], register[1...]);
        let controlledReference = register => Controlled reference(register[...0], register[1...]);
        let areEquivalent = CheckOperationsEquivalence(controlledOp, controlledReference, inputSize + 1);
        areEquivalent
    }

    operation CheckOperationsEquivalenceOnZeroState(
        op : (Qubit[] => Unit is Adj + Ctl),
        reference : (Qubit[] => Unit is Adj + Ctl),
        inputSize: Int)
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