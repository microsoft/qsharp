namespace Kata.Verification {
    import KatasUtils.*;

    operation AllBellStates_Reference(qs : Qubit[], index : Int) : Unit is Adj + Ctl {
        H(qs[0]);

        if index == 1 {
            Z(qs[0]);
        }
        if index == 2 {
            X(qs[1]);
        }
        if index == 3 {
            Z(qs[0]);
            X(qs[1]);
        }

        CNOT(qs[0], qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for index in 0..3 {
            Message($"Testing index = {index}...");
            if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                Kata.AllBellStates(_, index),
                AllBellStates_Reference(_, index),
                2
            ) {
                return false;
            }
        }

        return true;
    }
}
