namespace Kata {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation VerifyExercise() : Bool {
        let isCorrect = VerifyMultiQubitOperation(BellState, Kata.Solution.BellState);

        // Output different feedback to the user depending on whether the exercise was correct.
        use target = Qubit[2];
        if isCorrect {
            ShowEffectOnQuantumState(target, BellState);
        } else {
            ShowQuantumStateComparison(target, BellState, Kata.Solution.BellState);
        }

        isCorrect
    }
}