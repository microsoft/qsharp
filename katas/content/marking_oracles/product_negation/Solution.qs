namespace Kata {
    operation Oracle_ProductWithNegation(x : Qubit[], y : Qubit, r : Bool[]) : Unit is Adj + Ctl {
        for i in 0 .. Length(x) - 1 {
            if r[i] {
                CNOT(x[i], y);
            } else {
                ApplyControlledOnInt(0, X, [x[i]], y);
            }
        }
    }
}
