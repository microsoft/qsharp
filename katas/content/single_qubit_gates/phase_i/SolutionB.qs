namespace Kata {
    operation PhaseFlip (q : Qubit) : Unit is Adj + Ctl {
        open Microsoft.Quantum.Math;
        R1(0.5 * PI(), q);
    }
}