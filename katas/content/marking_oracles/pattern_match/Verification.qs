namespace Kata.Verification {
    import KatasUtils.*;

    function F_PatternMatching(args : Bool[], a : Int[], r : Bool[]) : Bool {
        for i in 0..Length(a) - 1 {
            if args[a[i]] != r[i] {
                return false;
            }
        }
        return true;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for (n, a, r) in [
            (2, [], []),
            (2, [1], [true]),
            (3, [0, 2], [false, true]),
            (4, [1, 3], [true, false]),
            (5, [0, 1, 4], [true, true, false])
        ] {
            if not CheckOracleImplementsFunction(n, Kata.Oracle_PatternMatching(_, _, a, r), F_PatternMatching(_, a, r)) {
                Message($"Test failed for n = {n}, a = {a}, r = {r}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
