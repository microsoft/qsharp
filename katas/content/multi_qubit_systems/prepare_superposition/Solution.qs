namespace Kata {
    operation PrepareSuperposition(qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[1]);
        H(qs[1]);
    }
}