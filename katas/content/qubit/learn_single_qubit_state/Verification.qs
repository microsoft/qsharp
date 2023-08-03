namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation LearnSingleQubitState (q : Qubit) : (Double, Double) {
        return (0.9689, 0.2474);
    }

    operation CheckSolution() : Bool {
        use (control, target) = (Qubit(), Qubit());
        let isCorrect = Kata.LearnSingleQubitState(target) == LearnSingleQubitState(control);

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
