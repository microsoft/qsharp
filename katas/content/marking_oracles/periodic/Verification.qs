namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    function PeriodicF(args : Bool[]) : Bool {
        let N = Length(args);
        for P in 1 .. N - 1 {
            if PeriodicGivenPeriodF(args, P) {
                return true;
            }
        }
        return false;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 2 .. 6 {
            if not CheckOracleImplementsFunction(n, Kata.PeriodicOracle, PeriodicF) {
                Message($"Test failed for n = {n}");
                return false;    
            }
        }

        Message("Correct!");
        true
    }  
}
