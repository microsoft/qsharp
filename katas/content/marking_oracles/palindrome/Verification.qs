namespace Kata.Verification {
    import KatasUtils.*;

    function F_Palindrome(args : Bool[]) : Bool {
        let N = Length(args);
        for i in 0..N / 2 - 1 {
            if args[i] != args[N - i - 1] {
                return false;
            }
        }
        return true;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 2..6 {
            if not CheckOracleImplementsFunction(n, Kata.Oracle_Palindrome, F_Palindrome) {
                Message($"Test failed for n = {n}");
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
