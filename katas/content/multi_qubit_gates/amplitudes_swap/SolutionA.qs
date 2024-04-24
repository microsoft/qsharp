namespace Kata {
    operation AmplitudesSwap (qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[1]);
        CNOT(qs[0], qs[1]);
    }
}