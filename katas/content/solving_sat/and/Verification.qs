namespace Kata.Verification {
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Arrays;

    function F_And(args : Bool[]) : Bool {
        return Count(x -> not x, args) == 0;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 1..5 {
            if not CheckOracleImplementsFunction(n, Kata.Oracle_And, F_And) {
                Message($"Test failed for n = {n}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
