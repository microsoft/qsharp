namespace Kata {
    operation TwoQubitGate3 (qs : Qubit[]) : Unit is Adj + Ctl {
        SWAP(qs[0], qs[1]);
    }
}