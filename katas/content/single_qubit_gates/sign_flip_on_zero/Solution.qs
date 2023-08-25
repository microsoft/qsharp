namespace Kata {
    operation SignFlipOnZero (q : Qubit) : Unit is Adj+Ctl {
        X(q);
        Z(q);
        X(q);
    }
}
