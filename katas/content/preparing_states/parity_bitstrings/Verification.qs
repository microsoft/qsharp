namespace Kata.Verification {
    import Std.Convert.*;
    import KatasUtils.*;

    operation AllStatesWithParitySuperposition_Reference(qs : Qubit[], parity : Int) : Unit is Adj + Ctl {
        if Length(qs) == 1 {
            if parity == 1 {
                X(qs[0]);
            }
        } else {
            H(qs[0]);
            ApplyControlledOnInt(0, AllStatesWithParitySuperposition_Reference, qs[0..0], (qs[1...], parity));
            ApplyControlledOnInt(1, AllStatesWithParitySuperposition_Reference, qs[0..0], (qs[1...], 1 - parity));
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for N in 2..5 {
            for parity in 0..1 {
                Message($"Testing for N = {N}, with parity: {parity}...");
                if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                    Kata.AllStatesWithParitySuperposition(_, parity),
                    AllStatesWithParitySuperposition_Reference(_, parity),
                    N
                ) {
                    return false;
                }
            }
        }

        return true;
    }
}
