namespace Kata {
    open Microsoft.Quantum.Math;
    operation GlobalPhaseChange (q : Qubit) : Unit is Adj + Ctl {
        Z(q);
        X(q);
        Z(q);
        X(q);
    }
}
