namespace Kata.Verification {
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Random;

    operation Or_Oracle_Reference(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        X(y);
        ApplyControlledOnInt(0, X, x, y);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for N in 1 .. 3 {
            let sol = ApplyOracle(_, Kata.Or_Oracle);
            let ref = ApplyOracle(_, Or_Oracle_Reference);
            let isCorrect = CheckOperationsEquivalenceStrict(sol, ref, N + 1);

            if not isCorrect {
                Message("Incorrect.");
                Message("Hint: examine how your solution transforms the given state and compare it with the expected " +
                    $"transformation for the {N}-bit oracle");
                use initial = Qubit[N + 1];
                PrepRandomState(initial[...N - 1]);
                ShowQuantumStateComparison(initial, sol, ref);
                ResetAll(initial);
                return false;
            }
        }
        Message("Correct!");
        true
    }
}
