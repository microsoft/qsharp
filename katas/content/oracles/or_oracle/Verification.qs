namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Random.*;

    operation Or_Oracle_Reference(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        X(y);
        ApplyControlledOnInt(0, X, x, y);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for N in 1..3 {
            let sol = ApplyOracle(_, Kata.Or_Oracle);
            let ref = ApplyOracle(_, Or_Oracle_Reference);
            let isCorrect = CheckOperationsAreEqualStrict(N + 1, sol, ref);

            if not isCorrect {
                Message("Incorrect.");
                Message("Hint: examine how your solution transforms the given state and compare it with the expected " +
                    $"transformation for the {N}-bit oracle");
                ShowQuantumStateComparison(N + 1, qs => PrepDemoState(qs[...N - 1]), sol, ref);
                return false;
            }
        }
        Message("Correct!");
        true
    }
}
