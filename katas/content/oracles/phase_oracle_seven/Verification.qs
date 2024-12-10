namespace Kata.Verification {
    import Std.Arrays.*;
    import KatasUtils.*;
    import Std.Math.*;

    operation IsSeven_PhaseOracle_Reference(x : Qubit[]) : Unit is Adj + Ctl {
        Controlled Z(Most(x), Tail(x));
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let N = 3;
        let isCorrect = CheckOperationsAreEqualStrict(
            3,
            Kata.IsSeven_PhaseOracle,
            IsSeven_PhaseOracle_Reference
        );
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
            Message("Hint: examine how your solution transforms the given state and compare it with the expected " +
                "transformation");
            ShowQuantumStateComparison(3, PrepDemoState, Kata.IsSeven_PhaseOracle, IsSeven_PhaseOracle_Reference);
        }
        isCorrect
    }
}
