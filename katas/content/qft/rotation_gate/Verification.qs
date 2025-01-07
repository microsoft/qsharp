namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Math.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for k in 0..10 {
            let solution = qs => Kata.Rotation(qs[0], k);
            let reference = qs => R1Frac(2, k, qs[0]);
            if not CheckOperationsAreEqualStrict(1, solution, reference) {
                Message($"Incorrect for k = {k}.");
                Message("Hint: examine the effect your solution has on the state 0.6|0〉 + 0.8|1〉 and compare it with the effect it " +
                    "is expected to have.");
                ShowQuantumStateComparison(1, qs => Ry(ArcTan2(0.8, 0.6) * 2.0, qs[0]), solution, reference);
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
