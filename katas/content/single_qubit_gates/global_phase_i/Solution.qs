namespace Kata {
    operation GlobalPhaseI(q : Qubit) : Unit is Adj + Ctl {
        Z(q);
        Y(q);
        X(q);
    }
}