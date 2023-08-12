namespace Kata {
    operation CompoundGate (qs : Qubit[]) : Unit is Adj + Ctl {
        S(qs[0]);
        I(qs[1]); // this line can be omitted, since it doesn't change the qubit state
        Y(qs[2]);
    }
}