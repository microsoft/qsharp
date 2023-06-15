namespace Kata {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation BellStateReference (qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        CNOT(qs[0], qs[1]);
    }

    operation VerifyExercise() : Bool {
        let op = BellState;
        let reference = BellStateReference;
        let isCorrect = VerifyMultiQubitOperation(op, reference);

        // Output different feedback to the user depending on whether the exercise was correct.
        use target = Qubit[2];
        if isCorrect {
            ShowEffectOnQuantumState(target, op);
        } else {
            ShowQuantumStateComparison(target, op, reference);
        }

        isCorrect
    }
}