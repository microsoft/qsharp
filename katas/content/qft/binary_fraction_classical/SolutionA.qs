namespace Kata {
    operation BinaryFractionClassical(q : Qubit, j : Bool[]) : Unit is Adj + Ctl {
        for ind in 0 .. Length(j) - 1 {
            if j[ind] {
                R1Frac(2, ind + 1, q);
            }
        }
    }
}