namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Math.*;
    operation ConditionalPhaseFlip(qs : Qubit[]) : Unit is Adj + Ctl {
        within {
            ApplyToEachA(X, qs);
        } apply {
            Controlled Z(qs[1...], qs[0]);
        }
        R(PauliI, 2.0 * PI(), qs[0]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for N in 2..4 {
            if not CheckOperationsAreEqualStrict(N, Kata.ConditionalPhaseFlip, ConditionalPhaseFlip) {
                Message("Incorrect.");
                Message("Hint: examine how your solution transforms the given state and compare it with the expected " +
                    $"transformation for the {N}-bit inputs");
                ShowQuantumStateComparison(N, PrepDemoState, Kata.ConditionalPhaseFlip, ConditionalPhaseFlip);
                return false;
            }
        }
        Message("Correct!");
        true
    }

}
