namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    function PatternMatchingF(args : Bool[], indices : Int[], pattern : Bool[]) : Bool {
        for i in 0 .. Length(indices) - 1 {
            if args[indices[i]] != pattern[i] {
                return false;
            }
        }
        return true;
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for (n, indices, pattern) in [
            (2, [], []),
            (2, [1], [true]),
            (3, [0, 2], [false, true]),
            (4, [1, 3], [true, false]),
            (5, [0, 1, 4], [true, true, false])
        ] {
            if not CheckOracleImplementsFunction(n, Kata.PatternMatchingOracle(_, _, indices, pattern), PatternMatchingF(_, indices, pattern)) {
                Message($"Test failed for n = {n}, indices = {indices}, pattern = {pattern}");
                return false;    
            }
        }

        Message("Correct!");
        true 
    }  
}
