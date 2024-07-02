namespace Kata.Verification {
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Arrays;

    function F_Majority(args : Bool[]) : Bool {
        let N = Length(args);
        return Count(x -> x, args) > (N - 1) / 2;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in [3, 5, 7] {
            if not CheckOracleImplementsFunction(n, Kata.Oracle_Majority, F_Majority) {
                Message($"Test failed for n = {n}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
