namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for (n, p, r) in [
            (2, 1, [true]),
            (3, 0, [false, true]),
            (4, 1, [true, true, false]),
            (5, 3, [false])
        ] {
            if not CheckOracleImplementsFunction(n, Kata.Oracle_ContainsSubstringAtPosition(_, _, r, p), F_ContainsSubstringAtPosition(_, r, p)) {
                Message($"Test failed for n = {n}, p = {p}, r = {r}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
