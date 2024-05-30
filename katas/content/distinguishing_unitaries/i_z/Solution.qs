namespace Kata {
    operation DistinguishIZ (unitary : (Qubit => Unit is Adj + Ctl)) : Int {
        use q = Qubit();
        H(q);
        unitary(q);
        return MResetX(q) == Zero ? 0 | 1;
    }
}
