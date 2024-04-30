namespace Kata {
    operation PhaseflipEncode (qs : Qubit[]) : Unit is Adj + Ctl {
        CNOT(qs[0], qs[1]);
        CNOT(qs[0], qs[2]);
        ApplyToEachCA(H, qs);
    }
}