namespace Kata {
    operation SignFlip (q : Qubit) : Unit is Adj + Ctl {
        Z(q);
    }
}