namespace Kata {
    operation BellState (qs : Qubit[]) : Unit is Adj {
        H(qs[0]);
        CNOT(qs[0], qs[1]);
    }
}