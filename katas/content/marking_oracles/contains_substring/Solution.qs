namespace Kata {
    operation Oracle_ContainsSubstring (x : Qubit[], y : Qubit, r : Bool[]) : Unit is Adj + Ctl {
        let N = Length(x);
        let K = Length(r);
        use aux = Qubit[N - K + 1];
        within {
            for P in 0 .. N - K {
                Oracle_ContainsSubstringAtPosition(x, aux[P], r, P);
            }
        } apply {
            ApplyControlledOnInt(0, X, aux, y);
            X(y);
        }
    }

    operation Oracle_ContainsSubstringAtPosition (x : Qubit[], y : Qubit, r : Bool[], p : Int) : Unit is Adj + Ctl {
        ApplyControlledOnBitString(r, X, x[p .. p + Length(r) - 1], y);
    }    
}
