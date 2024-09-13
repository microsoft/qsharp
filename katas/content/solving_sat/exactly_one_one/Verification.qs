namespace Kata.Verification {
    import Std.Arrays.*;
    import Std.Katas.*;

    function F_Exactly1One (args : Bool[]) : Bool {
        return Count(x -> x, args) == 1;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        if not CheckOracleImplementsFunction(3, Kata.Oracle_Exactly1One, F_Exactly1One) {
            return false;
        }

        Message("Correct!");
        true
    }
}
