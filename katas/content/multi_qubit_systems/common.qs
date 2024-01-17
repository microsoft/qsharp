namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    operation CheckEqualOnZeroState(
        testImpl : (Qubit[] => Unit is Adj + Ctl),
        refImpl : (Qubit[] => Unit is Adj + Ctl)) : Bool {

        let isCorrect = CheckOperationsEquivalenceOnZeroStateStrict(testImpl, refImpl, 2);

        // Output different feedback to the user depending on whether the exercise was correct.
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
            use target = Qubit[2]; // |00〉
            ShowQuantumStateComparison(target, testImpl, refImpl);
            ResetAll(target);
        }
        isCorrect
    }

}
