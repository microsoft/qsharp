namespace Kata.Verification {
    import KatasUtils.*;

    function F_Parity(x : Bool[]) : Bool {
        mutable parity = false;
        for xi in x {
            if xi {
                set parity = not parity;
            }
        }
        parity
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 1..5 {
            if not CheckOracleImplementsFunction(n, Kata.Oracle_Parity, F_Parity) {
                return false;
            }
        }

        Message("Correct!");
        true
    }
}
