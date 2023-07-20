namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation LearnBasisStateAmplitudes (qs : Qubit[]) : (Double, Double) {
        return (0.3821, 0.339);
    }

    operation CheckSolution() : Bool {
        let isCorrect = VerifySingleQubitOperation(Kata.LearnBasisStateAmplitudes, LearnBasisStateAmplitudes);

        // Output different feedback to the user depending on whether the exercise was correct.
        use target = Qubit[1];
        let op = register => Kata.LearnBasisStateAmplitudes(register[0]);
        let reference = register => LearnBasisStateAmplitudes(register[0]);
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
