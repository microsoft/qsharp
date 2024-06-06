namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    function  ContainsSubstringAtPositionF(args : Bool[], pattern : Bool[], P : Int) : Bool {
        for i in 0 .. Length(pattern) - 1 {
            if pattern[i] != args[i + P] {
                return false;
            }
        }
        return true;
    }    

    @EntryPoint()
    operation CheckSolution() : Bool {
        for (n, p, pattern) in [
            (2, 1, [true]),
            (3, 0, [false, true]),
            (4, 1, [true, true, false]),
            (5, 3, [false])
        ] {
            if not CheckOracleImplementsFunction(n, Kata.ContainsSubstringAtPositionOracle(_, _, pattern, p), ContainsSubstringAtPositionF(_, pattern, p)) {
                Message($"Test failed for n = {n}, p = {p}, pattern = {pattern}");
                return false;    
            }
        }

        Message("Correct!");
        true 
    }  
}
