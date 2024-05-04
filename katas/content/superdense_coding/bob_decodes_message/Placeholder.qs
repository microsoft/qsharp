namespace Kata {
    operation DecodeMessageFromQubits(qAlice : Qubit, qBob: Qubit) : (Bool, Bool) {
        // Implement your solution here...

        // Returning the qubit states after measuring
        return (MResetZ(qAlice) == One, MResetZ(qBob) == One);
    }
}
