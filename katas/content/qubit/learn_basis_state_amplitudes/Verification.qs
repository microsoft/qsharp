namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation CheckSolution() : Bool {
        use target = Qubit[2];
        let isCorrect = Kata.LearnBasisStateAmplitudes(target) ==  (0.3821, 0.339);

        // Output different feedback to the user depending on whether the exercise was correct.
        if isCorrect {
            Message("Correct!");
            Message("The solution was correct for all test cases.");
        } else {
            Message("Incorrect.");
            Message("The solution was incorrect for at least one test case.");
        }
        isCorrect
    }
}
