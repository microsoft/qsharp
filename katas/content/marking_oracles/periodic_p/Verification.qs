namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 2..6 {
            for p in 2..n - 1 {
                if not CheckOracleImplementsFunction(n, Kata.Oracle_PeriodicGivenPeriod(_, _, p), F_PeriodicGivenPeriod(_, p)) {
                    Message($"Test failed for n = {n}, p = {p}");
                    return false;
                }
            }
        }

        Message("Correct!");
        true
    }
}
