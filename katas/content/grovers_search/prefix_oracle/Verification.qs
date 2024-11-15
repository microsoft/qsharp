namespace Kata.Verification {
    import KatasUtils.*;

    function F_StartsWith(args : Bool[], p : Bool[]) : Bool {
        for i in 0..Length(p) - 1 {
            if p[i] != args[i] {
                return false;
            }
        }
        return true;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for (n, p) in [
            (2, []),
            (2, [true]),
            (2, [true, false]),
            (3, [false, true]),
            (4, [true, true, false]),
            (5, [false])
        ] {
            if not CheckOracleImplementsFunction(n, Kata.Oracle_StartsWith(_, _, p), F_StartsWith(_, p)) {
                Message($"Test failed for N = {n}, p = {p}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
