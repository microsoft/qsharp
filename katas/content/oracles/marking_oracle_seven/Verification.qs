namespace Kata.Verification {
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Math;

    operation IsSeven_MarkingOracle_Reference(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        Controlled X(x, y);
    }

    // ------------------------------------------------------
    @EntryPoint()
    operation CheckSolution () : Bool {
        let N = 3;
        let sol = ApplyOracle(_, Kata.IsSeven_MarkingOracle);
        let ref = ApplyOracle(_, IsSeven_MarkingOracle_Reference);
        let isCorrect = CheckOperationsEquivalenceStrict(sol, ref, N + 1);
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
            Message("Hint: examine how your solution transforms the given state and compare it with the expected " +
                "transformation");
            use initial = Qubit[4]; // |000〉
            PrepRandomState(initial[...2]);
            ShowQuantumStateComparison(initial, sol, ref);
            ResetAll(initial);
        }
        isCorrect
    }
}
