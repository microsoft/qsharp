namespace Kata.Verification {
    import KatasUtils.*;

    function F_ContainsSubstring(args : Bool[], r : Bool[]) : Bool {
        let N = Length(args);
        let K = Length(r);
        for P in 0..N - K {
            if F_ContainsSubstringAtPosition(args, r, P) {
                return true;
            }
        }
        return false;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for (n, r) in [
            (2, [true]),
            (3, [false, true]),
            (4, [true, true, false]),
            (5, [false, false])
        ] {
            if not CheckOracleImplementsFunction(n, Kata.Oracle_ContainsSubstring(_, _, r), F_ContainsSubstring(_, r)) {
                Message($"Test failed for n = {n}, r = {r}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
