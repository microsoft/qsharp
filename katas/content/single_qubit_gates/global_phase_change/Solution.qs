namespace Kata {
    open Microsoft.Quantum.Math;
    operation GlobalPhaseChange (q : Qubit) : Unit is Adj + Ctl {
        R(PauliI, 2.0 * PI(), q);
    }
}