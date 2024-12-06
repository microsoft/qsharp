namespace Kata {
    operation PhaseFlip(q : Qubit) : Unit is Adj + Ctl {
        import Std.Math.*;
        R1(0.5 * PI(), q);
    }
}
