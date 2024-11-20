namespace Kata.Verification {
    import Std.Arrays.*;
    import Std.Convert.*;
    import KatasUtils.*;
    import Std.Math.*;

    operation BinaryFractionClassical_Alternative(q : Qubit, j : Bool[]) : Unit is Adj + Ctl {
        // Convert the number to an integer and apply a single R1 rotation
        R1(2.0 * PI() * IntAsDouble(BoolArrayAsInt(Reversed(j))) / IntAsDouble(1 <<< Length(j)), q);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 1..5 {
            for exp in 0..(1 <<< n) - 1 {
                let j = Reversed(IntAsBoolArray(exp, n));
                let solution = qs => Kata.BinaryFractionClassical(qs[0], j);
                let reference = qs => BinaryFractionClassical_Alternative(qs[0], j);
                if not CheckOperationsAreEqualStrict(1, solution, reference) {
                    Message($"Incorrect for j = {j}.");
                    Message("Hint: examine the effect your solution has on the state 0.6|0〉 + 0.8|1〉 and compare it with the effect it " +
                        "is expected to have.");
                    ShowQuantumStateComparison(1, qs => Ry(ArcTan2(0.8, 0.6) * 2.0, qs[0]), solution, reference);
                    return false;
                }
            }
        }

        Message("Correct!");
        true
    }
}
