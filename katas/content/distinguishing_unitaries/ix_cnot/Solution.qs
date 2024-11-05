namespace Kata {
    operation DistinguishIXfromCNOT (unitary : (Qubit[] => Unit is Adj + Ctl)) : Int {
        use qs = Qubit[2];
        unitary(qs);
        return MResetZ(qs[1]) == One ? 0 | 1;
    }
}
