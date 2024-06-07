namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    function  ContainsSubstringAtPositionF(args : Bool[], r : Bool[], p : Int) : Bool {
        for i in 0 .. Length(r) - 1 {
            if r[i] != args[i + p] {
                return false;
            }
        }
        return true;
    }    

    @EntryPoint()
    operation CheckSolution() : Bool {
        for (n, p, r) in [
            (2, 1, [true]),
            (3, 0, [false, true]),
            (4, 1, [true, true, false]),
            (5, 3, [false])
        ] {
            if not CheckOracleImplementsFunction(n, Kata.ContainsSubstringAtPositionOracle(_, _, r, p), ContainsSubstringAtPositionF(_, r, p)) {
                Message($"Test failed for n = {n}, p = {p}, r = {r}");
                return false;    
            }
        }

        Message("Correct!");
        true 
    }  
}
