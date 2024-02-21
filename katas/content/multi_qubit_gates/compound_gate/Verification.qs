namespace Kata.Verification {
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Math;

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
            Message("Incorrect.");
            Message("Hint: examine how your solution transforms the given state and compare it with the expected " +
                "transformation");
            use initial = Qubit[3]; // |000〉
            Ry(ArcTan2(0.8, 0.6) * 2.0, initial[0]);
            Ry(ArcTan2(0.7, 0.4) * 2.0, initial[1]);
            Ry(ArcTan2(0.6, 0.5) * 2.0, initial[2]);
            ShowQuantumStateComparison(initial, solution, reference);
            ResetAll(initial);
        }

        isCorrect
    }
}