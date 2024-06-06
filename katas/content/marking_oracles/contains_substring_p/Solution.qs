namespace Kata {
    operation ContainsSubstringAtPositionOracle (input : Qubit[], target : Qubit, pattern : Bool[], P : Int) : Unit is Adj + Ctl {
        ApplyControlledOnBitString(pattern, X, input[P .. P + Length(pattern) - 1], target);
    }
}
