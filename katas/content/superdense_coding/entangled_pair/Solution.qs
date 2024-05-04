namespace Kata {
    operation CreateEntangledPair(qAlice : Qubit, qBob : Qubit) : Unit is Adj {
        // Implement your solution here...
        H(qAlice);
        CNOT(qAlice, qBob);
    }
}
