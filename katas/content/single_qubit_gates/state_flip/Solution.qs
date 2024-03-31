namespace Kata {
    operation StateFlip (q : Qubit) : Unit is Adj + Ctl {
        X(q);
    }
}