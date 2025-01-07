namespace Kata.Verification {
    import KatasUtils.*;
    import Std.Arrays.*;

    function F_ColorEquality(args : Bool[]) : Bool {
        // Check that the first half equals the second half
        let nBits = Length(args) / 2;
        for (f, s) in Zipped(args[...nBits - 1], args[nBits...]) {
            if f != s {
                return false;
            }
        }
        return true;
    }

    operation Oracle_ColorEquality_Wrapper(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        let nBits = Length(x) / 2;
        Kata.Oracle_ColorEquality(x[...nBits - 1], x[nBits...], y);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for nBits in 1..3 {
            if not CheckOracleImplementsFunction(2 * nBits, Oracle_ColorEquality_Wrapper, F_ColorEquality) {
                Message($"Test failed for nBits = {nBits}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
