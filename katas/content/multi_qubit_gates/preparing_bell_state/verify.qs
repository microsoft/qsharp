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

        if CheckZero(targetRegister[0]) and CheckZero(targetRegister[1]) {
            task(targetRegister);
            DumpMachine();
            return true;
        }

        Reset(targetRegister[0]);
        Reset(targetRegister[1]);

        // Use DumpMachine to display actual vs desired state.
        task(targetRegister);
        DumpMachine();
        Reset(targetRegister[0]);
        Reset(targetRegister[1]);
        taskRef(targetRegister);
        DumpMachine();
        return false;
    }
}