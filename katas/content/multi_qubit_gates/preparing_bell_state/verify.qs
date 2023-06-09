namespace Kata {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;

    operation BellStateReference (qs : Qubit[]) : Unit is Adj {
        H(qs[0]);
        CNOT(qs[0], qs[1]);
    }

    operation VerifyExercise() : Bool {
        let task = BellState;
        let taskRef = BellStateReference;

        mutable isCorrect = false;
        {
            use targetRegister = Qubit[2];
            task(targetRegister);
            Adjoint taskRef(targetRegister);
            set isCorrect = CheckAllZero(targetRegister);
            ResetAll(targetRegister);
        }

        if isCorrect {
            use targetRegister = Qubit[2];
            task(targetRegister);
            Message("Qubits state after setting them into a Bell state:");
            DumpMachine();
            ResetAll(targetRegister);
        } else {
            {
                use expected = Qubit[2];
                taskRef(expected);
                Message("Expected qubits state:");
                DumpMachine();
                ResetAll(expected);
            }

            {
                use actual = Qubit[2];
                task(actual);
                Message("Actual qubits state:");
                DumpMachine();
                ResetAll(actual);
            }
        }

        return isCorrect;
    }
}