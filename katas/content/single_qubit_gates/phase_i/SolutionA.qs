namespace Kata {
    operation PhaseFlip (q : Qubit) : Unit is Adj + Ctl {
        S(q);
    }
}