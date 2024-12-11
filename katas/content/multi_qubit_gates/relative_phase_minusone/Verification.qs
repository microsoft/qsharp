namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Diagnostics.*;

    operation PrepareState(qs : Qubit[]) : Unit is Adj + Ctl {
        ApplyToEachCA(H, qs);
    }

    operation RelativePhaseMinusOne(qs : Qubit[]) : Unit is Adj + Ctl {
        CZ(qs[0], qs[1]);
    }


    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = CheckOperationsEquivalenceOnInitialStateStrict(
            PrepareState,
            Kata.RelativePhaseMinusOne,
            RelativePhaseMinusOne,
            2
        );
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect");
            ShowQuantumStateComparison(2, PrepareState, Kata.RelativePhaseMinusOne, RelativePhaseMinusOne);
        }

        return isCorrect;
    }
}
