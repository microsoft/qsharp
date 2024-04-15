namespace Kata {
    operation TwoQubitGate2 (qs : Qubit[]) : Unit is Adj + Ctl {
        CZ(qs[0], qs[1]);
    }
}