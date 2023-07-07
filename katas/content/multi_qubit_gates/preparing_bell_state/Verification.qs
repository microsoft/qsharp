namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation BellState (qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        CNOT(qs[0], qs[1]);
    }

    operation CheckSolution() : Bool {
        let isCorrect = VerifyMultiQubitOperation(Kata.BellState, BellState);

        // Output different feedback to the user depending on whether the exercise was correct.
        use target = Qubit[2];
        if isCorrect {
            ShowEffectOnQuantumState(target, Kata.BellState);
        } else {
            ShowQuantumStateComparison(target, Kata.BellState, BellState);
        }

        isCorrect
    }
}