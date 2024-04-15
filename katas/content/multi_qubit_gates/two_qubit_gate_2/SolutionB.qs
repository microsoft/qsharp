namespace Kata {
    operation TwoQubitGate2 (qs : Qubit[]) : Unit is Adj + Ctl {
        Controlled Z([qs[0]], qs[1]);
    }
}