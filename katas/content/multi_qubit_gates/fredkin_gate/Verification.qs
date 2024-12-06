namespace Kata.Verification {
    import KatasUtils.*;

    operation FredkinGate(qs : Qubit[]) : Unit is Adj + Ctl {
        Controlled SWAP([qs[0]], (qs[1], qs[2]));
    }

    operation CheckSolution() : Bool {
        let solution = Kata.FredkinGate;
        let reference = FredkinGate;
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
