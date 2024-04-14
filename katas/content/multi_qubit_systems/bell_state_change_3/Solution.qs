namespace Kata {
    operation BellStateChange3(qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[0]);
        Z(qs[0]);
    }
}