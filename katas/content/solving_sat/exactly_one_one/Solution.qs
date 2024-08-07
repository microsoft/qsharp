namespace Kata {
    operation Oracle_Exactly1One(x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        for i in 0 .. Length(x) - 1 {
            ApplyControlledOnInt(2 ^ i, X, x, y);
        }
    }        
}
