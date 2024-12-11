namespace Kata.Verification {
    import KatasUtils.*;

    function F_Kth_Bit(x : Bool[], k : Int) : Bool {
        x[k]
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for n in 3..5 {
            for k in 0..n - 1 {
                if not CheckOracleImplementsFunction(n, Kata.Oracle_Kth_Bit(_, _, k), F_Kth_Bit(_, k)) {
                    Message($"Test failed for n = {n}, k = {k}");
                    return false;
                }
            }
        }

        Message("Correct!");
        true
    }
}
