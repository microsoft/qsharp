namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;


    operation CheckSolution() : Bool {
        use target = Qubit[2];
        let isCorrect = Kata.HighProbabilityBasisStates(target) == [0, 2, 8, 9, 11, 15, 18, 20, 22, 25, 28];

        // Output different feedback to the user depending on whether the exercise was correct.
        if isCorrect {
            Message("Correct!");
            Message("The solution was correct for all test cases.");
        } else {
            Message("Incorrect.");
            Message("The solution was incorrect for at least one test case.");
        }
        ResetAll(target);
        isCorrect
    }
}
