namespace Kata {
    operation Rotation(q : Qubit, k : Int) : Unit is Adj + Ctl {
        R1Frac(2, k, q);
    }
}