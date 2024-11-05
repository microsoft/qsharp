namespace Kata {
    operation CNOTDirection (unitary : (Qubit[] => Unit is Adj + Ctl)) : Int {
        use qs = Qubit[2];
        within { X(qs[1]); }
        apply { unitary(qs); }
        return MResetZ(qs[0]) == Zero ? 0 | 1;
    }
}
