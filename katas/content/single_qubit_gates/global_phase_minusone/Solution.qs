namespace Kata {
    operation GlobalPhaseChange (q : Qubit) : Unit is Adj + Ctl {
        Z(q);
        X(q);
        Z(q);
        X(q);
    }
}
