namespace Kata {
    operation ToffoliGate (qs : Qubit[]) : Unit is Adj + Ctl {
        CCNOT(qs[0], qs[1], qs[2]);
    }
}