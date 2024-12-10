namespace Kata.Verification {
    import KatasUtils.*;

    operation ToffoliGate(qs : Qubit[]) : Unit is Adj + Ctl {
        CCNOT(qs[0], qs[1], qs[2]);
    }

    operation CheckSolution() : Bool {
        let solution = Kata.ToffoliGate;
        let reference = ToffoliGate;
        let isCorrect = CheckOperationsAreEqualStrict(3, solution, reference);

        // Output different feedback to the user depending on whether the solution was correct.
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
            Message("Hint: examine the state prepared by your solution and compare it with the state it " +
                "is expected to prepare.");
            ShowQuantumStateComparison(3, PrepDemoState, solution, reference);
        }

        isCorrect
    }
}
