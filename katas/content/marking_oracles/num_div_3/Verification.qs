namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Convert.*;

    function F_DivisibleBy3(args : Bool[]) : Bool {
        return BoolArrayAsInt(args) % 3 == 0;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 2..7 {
            if not CheckOracleImplementsFunction(n, Kata.Oracle_DivisibleBy3, F_DivisibleBy3) {
                Message($"Test failed for n = {n}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
