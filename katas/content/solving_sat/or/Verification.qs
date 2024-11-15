namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Arrays.*;

    function F_Or(args : Bool[]) : Bool {
        return Count(x -> x, args) > 0;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 1..5 {
            if not CheckOracleImplementsFunction(n, Kata.Oracle_Or, F_Or) {
                Message($"Test failed for n = {n}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
