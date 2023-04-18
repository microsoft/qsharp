namespace Kata {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;

    operation BellStateReference (qs : Qubit[]) : Unit is Adj {
        body ... {
            H(qs[0]);
            CNOT(qs[0], qs[1]);
        }
        adjoint ... {
            CNOT(qs[0], qs[1]);
            H(qs[0]);
        }
    }

    operation Verify() : Bool {
        let task = BellState;
        let taskRef = BellStateReference;

        use targetRegister = Qubit[2];

        task(targetRegister);
        Adjoint taskRef(targetRegister);

        if CheckAllZero(targetRegister) {
            task(targetRegister);
            DumpMachine();
            return true;
        }

        ResetAll(targetRegister);

        // Use DumpMachine to display actual vs desired state.
        task(targetRegister);
        DumpMachine();
        ResetAll(targetRegister);
        taskRef(targetRegister);
        DumpMachine();
        return false;
    }
}