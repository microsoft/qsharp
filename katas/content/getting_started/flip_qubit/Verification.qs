namespace Kata.Verification {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation FlipQubit(q : Qubit): Unit is Adj + Ctl {
        X(q);
    }

    operation CheckSolution() : Bool {
        let solution = register => Kata.FlipQubit(register[0]);
        let reference = register => FlipQubit(register[0]);
        let isCorrect = CheckOperationsEquivalence(solution, reference, 1);

        // Output different feedback to the user depending on whether the solution was correct.
        if isCorrect {
            Message("Correct!");
            Message("Congratulations! You have solved your first exercise.");
        } else {
            Message("Incorrect.");
            Message("Look out for hints when your solution is incorrect.");
            Message("Hint: examine the effect your solution has on the |0〉 state and compare it with the effect it " +
                "is expected to have.");
            use target = Qubit[1]; // |0〉
            ShowQuantumStateComparison(target, solution, reference);
            ResetAll(target);
        }
        isCorrect
    }
}
