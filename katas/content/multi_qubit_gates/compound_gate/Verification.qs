namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Katas;

    operation CompoundGate (qs : Qubit[]) : Unit is Adj + Ctl {
        S(qs[0]);
        I(qs[1]); // this line can be omitted, since it doesn't change the qubit state
        Y(qs[2]);
    }

    operation CheckSolution() : Bool {
        let solution = Kata.CompoundGate;
        let reference = CompoundGate;
        let isCorrect = CheckOperationsEquivalenceStrict(solution, reference, 3);

        // Output different feedback to the user depending on whether the solution was correct.
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect :(");
            Message("Hint: examine how your solution transforms the |000〉 state and compare it with the expected " +
                "transformation");
            use target = Qubit[3]; // |000〉
            ShowQuantumStateComparison(target, solution, reference);
            ResetAll(target);
        }

        isCorrect
    }
}