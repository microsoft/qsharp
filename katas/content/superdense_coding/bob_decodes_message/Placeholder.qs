namespace Kata {
    operation DecodeMessageFromQubits(qAlice : Qubit, qBob : Qubit) : (Bool, Bool) {
        // Implement your solution here...

        // Returning the qubit states after measuring
        return (MResetZ(qAlice) == One, MResetZ(qBob) == One);
    }

    // You might find this helper operation from an earlier task useful.
    operation CreateEntangledPair(qAlice : Qubit, qBob : Qubit) : Unit is Adj {
        H(qAlice);
        CNOT(qAlice, qBob);
    }
}
