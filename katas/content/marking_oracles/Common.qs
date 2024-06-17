namespace Kata.Verification {   
    function F_PeriodicGivenPeriod(args : Bool[], P : Int) : Bool {
        let N = Length(args);
        for i in 0 .. N - P - 1 {
            if args[i] != args[i + P] {
                return false;
            }
        }
        return true;
    }

    function  F_ContainsSubstringAtPosition(args : Bool[], r : Bool[], p : Int) : Bool {
        for i in 0 .. Length(r) - 1 {
            if r[i] != args[i + p] {
                return false;
            }
        }
        return true;
    }     
}
