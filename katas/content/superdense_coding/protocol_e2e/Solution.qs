namespace Kata {

    operation SuperdenseCodingProtocol(message : (Bool, Bool)) : (Bool, Bool) {
        use (qAlice, qBob) = (Qubit(), Qubit());

        CreateEntangledPair(qAlice, qBob);

        EncodeMessageInQubit(qAlice, message);

        return DecodeMessageFromQubits(qAlice, qBob);
    }

    operation CreateEntangledPair(qAlice : Qubit, qBob : Qubit) : Unit is Adj {
        H(qAlice);
        CNOT(qAlice, qBob);
    }

    operation EncodeMessageInQubit(qAlice : Qubit, message : (Bool, Bool)) : Unit {
        let (bit1, bit2) = message;

        if bit2 {
            X(qAlice);
        }

        if bit1 {
            Z(qAlice);
        }
    }

    operation DecodeMessageFromQubits(qAlice : Qubit, qBob : Qubit) : (Bool, Bool) {
        CNOT(qAlice, qBob);
        H(qAlice);
        return (MResetZ(qAlice) == One, MResetZ(qBob) == One);
    }
}
