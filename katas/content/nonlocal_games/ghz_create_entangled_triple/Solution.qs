namespace Kata {
    operation CreateEntangledTriple (qs : Qubit[]) : Unit is Adj {
        X(qs[0]);
        X(qs[1]);
        H(qs[0]);
        H(qs[1]);
        Controlled Z([qs[0]], qs[1]);
        ApplyControlledOnBitString([false, true], X, [qs[0], qs[1]], qs[2]);
        ApplyControlledOnBitString([true, false], X, [qs[0], qs[1]], qs[2]);
    }
}
