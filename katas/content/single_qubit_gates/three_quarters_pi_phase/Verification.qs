namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Diagnostics;

    operation ThreeQuartersPiPhase(q : Qubit) : Unit is Adj + Ctl {
        S(q);
        T(q);
    }

    operation CheckSolution() : Bool {
        let solution = register => Kata.ThreeQuartersPiPhase(register[0]);
        let reference = register => ThreeQuartersPiPhase(register[0]);
        let isCorrect = CheckOperationsEquivalenceStrict(solution, reference, 1);

        // Output different feedback to the user depending on whether the exercise was correct.
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
            Message("The solution was incorrect for at least one test case.");
            use target = Qubit[1];
            ShowQuantumStateComparison(target, solution, reference);
            ResetAll(target);
        }
        isCorrect
    }
}
