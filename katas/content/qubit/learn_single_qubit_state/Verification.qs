namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation LearnSingleQubitState (q : Qubit) : (Double, Double) {
        return (0.9689, 0.2474);
    }

    operation CheckSolution() : Bool {
        let isCorrect = VerifySingleQubitOperation(Kata.LearnSingleQubitState, LearnSingleQubitState);

        // Output different feedback to the user depending on whether the exercise was correct.
        use target = Qubit[1];
        let op = register => Kata.LearnSingleQubitState(register[0]);
        let reference = register => LearnSingleQubitState(register[0]);
        if isCorrect {
            Message("Correct!");
            Message("The solution was correct for all test cases.");
            ShowEffectOnQuantumState(target, op);
        } else {
            Message("Incorrect.");
            Message("The solution was incorrect for at least one test case.");
            ShowQuantumStateComparison(target, op, reference);
        }
        isCorrect
    }
}
