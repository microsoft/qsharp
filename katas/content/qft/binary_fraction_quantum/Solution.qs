namespace Kata {
    operation BinaryFractionQuantum(q : Qubit, j : Qubit[]) : Unit is Adj + Ctl {
        for ind in 0 .. Length(j) - 1 {
            Controlled R1Frac([j[ind]], (2, ind + 1, q));
        }
    }
}