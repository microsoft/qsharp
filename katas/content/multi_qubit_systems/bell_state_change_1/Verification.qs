namespace Kata.Verification {
    import Std.Diagnostics.*;
    import KatasUtils.*;

    operation PrepareBellState(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        CNOT(qs[0], qs[1]);
    }


    operation BellStateChange1_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        Z(qs[0]);
    }


    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = CheckOperationsEquivalenceOnInitialStateStrict(
            PrepareBellState,
            Kata.BellStateChange1,
            BellStateChange1_Reference,
            2
        );

        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect");
            ShowQuantumStateComparison(2, PrepareBellState, Kata.BellStateChange1, BellStateChange1_Reference);
        }

        return isCorrect;
    }
}
