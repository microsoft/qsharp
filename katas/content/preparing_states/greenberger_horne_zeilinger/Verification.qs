namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Arrays.*;

    operation GHZ_State_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);

        for q in Rest(qs) {
            CNOT(qs[0], q);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for i in 1..5 {
            Message($"Testing {i} qubit(s)...");
            if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                Kata.GHZ_State,
                GHZ_State_Reference,
                i
            ) {
                return false;
            }
        }

        return true;
    }
}
