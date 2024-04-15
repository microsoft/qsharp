namespace Kata {
    operation BellStateChange1 (qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[0]);
    }
}