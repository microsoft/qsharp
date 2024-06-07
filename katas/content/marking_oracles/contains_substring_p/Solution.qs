namespace Kata {
    operation ContainsSubstringAtPositionOracle (x : Qubit[], y : Qubit, r : Bool[], p : Int) : Unit is Adj + Ctl {
        ApplyControlledOnBitString(r, X, x[p .. p + Length(r) - 1], y);
    }
}
