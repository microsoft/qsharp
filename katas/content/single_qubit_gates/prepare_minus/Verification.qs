namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    operation PrepareMinus(q : Qubit) : Unit is Adj + Ctl {
        X(q);
        H(q);
    }

    operation CheckSolution() : Bool {
        let solution = register => Kata.PrepareMinus(register[0]);
        let reference = register => PrepareMinus(register[0]);
        let isCorrect = CheckOperationsEquivalenceOnZeroStateStrict(solution, reference, 1);

        // Output different feedback to the user depending on whether the exercise was correct.
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
            Message("Hint: examine the state prepared by your solution and compare it with the state it " +
                "is expected to prepare.");
            use initial = Qubit(); // |0〉
            ShowQuantumStateComparison([initial], solution, reference);
            Reset(initial);
        }
        isCorrect
    }
}
