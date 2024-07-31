namespace Kata {
    operation BinaryFractionQuantumInPlace(j : Qubit[]) : Unit is Adj + Ctl {
        H(j[0]);
        for ind in 1 .. Length(j) - 1 {
            Controlled R1Frac([j[ind]], (2, ind + 1, j[0]));
        }
    }
}