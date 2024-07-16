namespace Kata {
    operation Oracle_StartsWith(x : Qubit[], y : Qubit, p : Bool[]) : Unit is Adj + Ctl {
        ApplyControlledOnBitString(p, X, x[... Length(p) - 1], y);
    }
}
