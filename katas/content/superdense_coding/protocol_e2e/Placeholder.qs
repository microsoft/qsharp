namespace Kata {
    operation SuperdenseCodingProtocol(message : (Bool, Bool)) : (Bool, Bool) {
        // Implement your solution here...

        return (false, false);
    }

    // You might find these helper operations from earlier tasks useful.
    operation CreateEntangledPair(qAlice : Qubit, qBob : Qubit) : Unit is Adj {
        H(qAlice);
        CNOT(qAlice, qBob);
    }

    // You might find this helper operation from an earlier task useful.
    operation EncodeMessageInQubit(qAlice : Qubit, message : (Bool, Bool)) : Unit {
        let (cbit1, cbit2) = message;

        if cbit2 {
            X(qAlice);
        }

        if cbit1 {
            Z(qAlice);
        }
    }

    // You might find this helper operation from an earlier task useful.
    operation DecodeMessageFromQubits(qAlice : Qubit, qBob : Qubit) : (Bool, Bool) {
        CNOT(qAlice, qBob);
        H(qAlice);
        return (MResetZ(qAlice) == One, MResetZ(qBob) == One);
    }
}
