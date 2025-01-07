namespace Kata.Verification {
    import KatasUtils.*;

    function F_Periodic(args : Bool[]) : Bool {
        let N = Length(args);
        for P in 1..N - 1 {
            if F_PeriodicGivenPeriod(args, P) {
                return true;
            }
        }
        return false;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 2..6 {
            if not CheckOracleImplementsFunction(n, Kata.Oracle_Periodic, F_Periodic) {
                Message($"Test failed for n = {n}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
