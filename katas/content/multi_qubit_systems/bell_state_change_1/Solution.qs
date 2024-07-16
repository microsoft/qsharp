namespace Kata {
    operation BellStateChange1 (qs : Qubit[]) : Unit is Adj + Ctl {
        Z(qs[0]);
    }
}