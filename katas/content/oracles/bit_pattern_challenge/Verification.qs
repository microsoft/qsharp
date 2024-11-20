namespace Kata.Verification {
    import Std.Arrays.*;
    import Std.Convert.*;
    import KatasUtils.*;

    operation ArbitraryBitPattern_Oracle_Challenge_Reference(x : Qubit[], pattern : Bool[]) : Unit is Adj + Ctl {
        within {
            for i in IndexRange(x) {
                if not pattern[i] {
                    X(x[i]);
                }
            }
        } apply {
            Controlled Z(Most(x), Tail(x));
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for N in 1..3 {
            for k in 0..2^N -1 {
                let pattern = IntAsBoolArray(k, N);

                let sol = Kata.ArbitraryBitPattern_Oracle_Challenge(_, pattern);
                let ref = ArbitraryBitPattern_Oracle_Challenge_Reference(_, pattern);
                let isCorrect = CheckOperationsAreEqualStrict(N, sol, ref);

                if not isCorrect {
                    Message("Incorrect.");
                    Message("Hint: examine how your solution transforms the given state and compare it with the expected " +
                        $"transformation for the {N}-bit oracle for pattern = {pattern}");
                    ShowQuantumStateComparison(N, PrepDemoState, sol, ref);
                    return false;
                }
            }
        }
        Message("All tests passed.");
        true
    }
}
