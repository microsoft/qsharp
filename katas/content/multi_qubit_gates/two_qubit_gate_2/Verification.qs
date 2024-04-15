namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    operation  TwoQubitGate2 (qs : Qubit[]) : Unit is Adj + Ctl {
        CZ(qs[0], qs[1]);
    }

    operation CheckSolution() : Bool {
        let solution = Kata. TwoQubitGate2;
        let reference =  TwoQubitGate2;
        let isCorrect = CheckOperationsEquivalence(solution, reference, 2);

        // Output different feedback to the user depending on whether the solution was correct.
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
            Message("Hint: examine the state prepared by your solution and compare it with the state it " +
                "is expected to prepare.");
            ShowQuantumStateComparison(2, (qs => ()), solution, reference);
        }

        isCorrect
    }
}