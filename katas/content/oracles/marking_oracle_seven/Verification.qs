namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Math.*;

    operation IsSeven_MarkingOracle_Reference(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        Controlled X(x, y);
    }

    // ------------------------------------------------------
    @EntryPoint()
    operation CheckSolution() : Bool {
        let N = 3;
        let sol = ApplyOracle(_, Kata.IsSeven_MarkingOracle);
        let ref = ApplyOracle(_, IsSeven_MarkingOracle_Reference);
        let isCorrect = CheckOperationsAreEqualStrict(N + 1, sol, ref);
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
            Message("Hint: examine how your solution transforms the given state and compare it with the expected " +
                "transformation");
            ShowQuantumStateComparison(4, qs => PrepDemoState(qs[...2]), sol, ref);
        }
        isCorrect
    }
}
