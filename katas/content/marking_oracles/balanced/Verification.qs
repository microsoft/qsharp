namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Arrays.*;

    function F_Balanced(args : Bool[]) : Bool {
        return Count(x -> x, args) == Length(args) / 2;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 2..2..6 {
            if not CheckOracleImplementsFunction(n, Kata.Oracle_Balanced, F_Balanced) {
                Message($"Test failed for n = {n}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
