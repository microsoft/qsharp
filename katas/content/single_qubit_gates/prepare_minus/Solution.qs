namespace Kata {
    operation PrepareMinus (q : Qubit) : Unit is Adj+Ctl {
        X(q);
        H(q);
    }
}
