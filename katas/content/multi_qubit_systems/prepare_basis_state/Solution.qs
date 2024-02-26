namespace Kata {
    operation PrepareBasisState(qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[0]);
        X(qs[1]);
    }
}