namespace Kata {
    operation DistinguishZfromMinusZ (unitary : (Qubit => Unit is Adj + Ctl)) : Int {
        use qs = Qubit[2];
        H(qs[0]);
        Controlled unitary(qs[0..0], qs[1]);
        return MResetX(qs[0]) == Zero ? 0 | 1;
    } 
}
