namespace Kata {
    operation TwoQubitGate1 (qs : Qubit[]) : Unit is Adj + Ctl {
        CNOT(qs[0], qs[1]);
    }
}