namespace Kata {
    operation TwoQubitGate3 (qs : Qubit[]) : Unit is Adj + Ctl {
        CNOT(qs[0], qs[1]);
        CNOT(qs[1], qs[0]);
        CNOT(qs[0], qs[1]);
    }
}