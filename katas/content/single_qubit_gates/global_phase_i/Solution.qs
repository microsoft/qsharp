namespace Kata.Solution {
    operation GlobalPhaseI(q : Qubit) : Unit is Adj + Ctl {
        X(q);
        Z(q);
        Y(q);
    }
}