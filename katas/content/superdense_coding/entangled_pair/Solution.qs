namespace Kata {
    operation CreateEntangledPair(qs : Qubit[]) : Unit is Adj {
        H(qs[0]);
        CNOT(qs[0], qs[1]);
    }
}
