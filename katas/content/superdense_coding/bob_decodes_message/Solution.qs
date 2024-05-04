namespace Kata {
    operation DecodeMessageFromQubits(qAlice : Qubit, qBob : Qubit) : (Bool, Bool) {
        Adjoint CreateEntangledPair(qAlice, qBob);

        // Returning the qubit states after measuring
        return (MResetZ(qAlice) == One, MResetZ(qBob) == One);
    }

    operation CreateEntangledPair(qAlice : Qubit, qBob : Qubit) : Unit is Adj {
        H(qAlice);
        CNOT(qAlice, qBob);
    }

}
