namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation BellState (qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        CNOT(qs[0], qs[1]);
    }

    operation CheckSolution() : Bool {
        let solution = Kata.BellState;
        let reference = BellState;
        let isCorrect = CheckOperationsEquivalenceOnZeroState(solution, reference, 2);

        // Output different feedback to the user depending on whether the solution was correct.
        use target = Qubit[2]; // |00〉
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect :(");
            Message("Hint: examine how your solution transforms the |00〉 state and compare it with the expected " +
                "transformation");
            ShowQuantumStateComparison(target, solution, reference);
        }

        isCorrect
    }
}