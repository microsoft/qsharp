// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Katas {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;

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

    /// # Summary
    /// Verifies that an operation is equivalent to a reference operation.
    operation VerifyMultiQubitOperation(
        unitary : (Qubit[] => Unit is Adj + Ctl),
        reference : (Qubit[] => Unit is Adj + Ctl))
    : Bool {
        use targetRegister = Qubit[2];
        unitary(targetRegister);
        Adjoint reference(targetRegister);
        let isCorrect = CheckAllZero(targetRegister);
        ResetAll(targetRegister);
        isCorrect
    }

    /// # Summary
    /// Verifies that an operation is equivalent to a reference operation.
    operation VerifySingleQubitOperation(
        op : (Qubit => Unit is Adj + Ctl),
        reference : (Qubit => Unit is Adj + Ctl))
    : Bool {
        use (control, target) = (Qubit(), Qubit());
        within {
            H(control);
        }
        apply {
            Controlled op([control], target);
            Adjoint Controlled reference([control], target);
        }
        let isCorrect = CheckAllZero([control, target]);
        ResetAll([control, target]);

        isCorrect
    }
}