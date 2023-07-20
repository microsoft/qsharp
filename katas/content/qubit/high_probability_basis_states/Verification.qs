namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation HighProbabilityBasisStates (qs : Qubit[]) : Int[] {
        return [0, 2, 8, 9, 11, 15, 18, 20, 22, 25, 28];
    }

    operation CheckSolution() : Bool {
        let isCorrect = VerifySingleQubitOperation(Kata.HighProbabilityBasisStates, HighProbabilityBasisStates);

        // Output different feedback to the user depending on whether the exercise was correct.
        use target = Qubit[1];
        let op = register => Kata.HighProbabilityBasisStates(register[0]);
        let reference = register => HighProbabilityBasisStates(register[0]);
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
