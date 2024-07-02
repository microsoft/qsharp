namespace Kata {
    operation DistinguishCNOTfromSWAP (unitary : (Qubit[] => Unit is Adj + Ctl)) : Int {
        use qs = Qubit[2];
        X(qs[1]);
        unitary(qs);
        Reset(qs[1]);
        return MResetZ(qs[0]) == Zero ? 0 | 1;
    }
}
