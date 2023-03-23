operation GlobalPhaseI(q : Qubit) : Unit is Adj + Ctl {
    body ... {
        X(q);
        Z(q);
        Y(q);
    }
    adjoint ... {
        Y(q);
        Z(q);
        X(q);
    } 
}