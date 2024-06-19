namespace Kata.Verification {
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Arrays;

    function F_BitSumDivisibleBy3(args : Bool[]) : Bool {
        return Count(x -> x, args) % 3 == 0;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 3 .. 6 {
            if not CheckOracleImplementsFunction(n, Kata.Oracle_BitSumDivisibleBy3, F_BitSumDivisibleBy3) {
                Message($"Test failed for n = {n}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
