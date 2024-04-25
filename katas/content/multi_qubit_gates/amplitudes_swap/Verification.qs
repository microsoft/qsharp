namespace Kata.Verification {
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Canon;

    operation  AmplitudesSwap (qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[1]);
        CNOT(qs[0], qs[1]);
    }

    operation CheckSolution() : Bool {
        let solution = Kata. AmplitudesSwap;
        let reference =  AmplitudesSwap;
        let isCorrect = CheckOperationsEquivalence(solution, reference, 2);

        // Output different feedback to the user depending on whether the solution was correct.
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
            Message("Hint: examine the state prepared by your solution and compare it with the state it " +
                "is expected to prepare.");
            ShowQuantumStateComparison(2, PrepRandomState, solution, reference);
        }

        isCorrect
    }
}