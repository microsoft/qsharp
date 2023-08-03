namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation ApplyY(q : Qubit) : Unit is Adj + Ctl {
        Y(q);
    }

    operation CheckSolution() : Bool {
        let solution = register => Kata.ApplyY(register[0]);
        let reference = register => ApplyY(register[0]);
        let isCorrect = CheckOperationsEquivalenceStrict(solution, reference, 1);

        // Output different feedback to the user depending on whether the solution was correct.
        use target = Qubit[1]; // |0〉
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
            Message("Hint: examine the effect your solution has on the |0〉 state and compare it with the effect it " +
                "is expected to have.");
            ShowQuantumStateComparison(target, solution, reference);
        }
        ResetAll(target);
        isCorrect
    }
}
