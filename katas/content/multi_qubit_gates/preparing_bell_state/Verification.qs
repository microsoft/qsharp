namespace Kata.Verification {
    import KatasUtils.*;

    operation BellState(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        CNOT(qs[0], qs[1]);
    }

    operation CheckSolution() : Bool {
        let solution = Kata.BellState;
        let reference = BellState;
        let isCorrect = CheckOperationsEquivalenceOnZeroState(solution, reference, 2);

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
