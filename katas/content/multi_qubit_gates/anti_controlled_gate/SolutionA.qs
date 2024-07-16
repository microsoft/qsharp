namespace Kata {
    operation AntiControlledGate (qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[1]);
        CNOT(qs[0], qs[1]);
    }
}