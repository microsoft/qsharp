namespace Kata {
    operation PrepareWithReal(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        X(qs[1]);
        H(qs[1]);
    }
}