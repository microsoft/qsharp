namespace Kata.Verification {
    import KatasUtils.*;

    operation AntiControlledGate(qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[1]);
        CNOT(qs[0], qs[1]);
    }

    operation CheckSolution() : Bool {
        let solution = Kata.AntiControlledGate;
        let reference = AntiControlledGate;
        let isCorrect = CheckOperationsAreEqualStrict(2, solution, reference);

        // Output different feedback to the user depending on whether the solution was correct.
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
            Message("Hint: examine the state prepared by your solution and compare it with the state it " +
                "is expected to prepare.");	
            ShowQuantumStateComparison(2, PrepDemoState, solution, reference);
        }

        isCorrect
    }
}
