namespace Kata {
    operation Oracle_ColorEquality(x0 : Qubit[], x1 : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        within {
            for i in 0..Length(x0) - 1 {
                CNOT(x0[i], x1[i]);
            }
        } apply {
            ApplyControlledOnInt(0, X, x1, y);
        }
    }
}
