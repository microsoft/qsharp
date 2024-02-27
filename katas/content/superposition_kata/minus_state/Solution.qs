namespace Kata {
    operation MinusState(q : Qubit) : Unit is Adj + Ctl {
        X(q);
        H(q);
    }
}
