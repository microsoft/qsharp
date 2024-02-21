namespace Kata {
    operation PrepareWithComplex(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        H(qs[1]);
        S(qs[0]);
        T(qs[1]);
    }
}