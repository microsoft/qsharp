namespace Kata {
    operation BellStateChange2 (qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[0]);
    }
}