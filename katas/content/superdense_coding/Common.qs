namespace Kata.Verification {
    operation CreateEntangledPairWrapper_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        let (qAlice, qBob) = (qs[0], qs[1]);
        H(qAlice);
        CNOT(qAlice, qBob);
    }

    operation EncodeMessageInQubit_Reference(qAlice : Qubit, message : (Bool, Bool)) : Unit {
        let (bit1, bit2) = message;

        if bit1 {
            Z(qAlice);
        }

        if bit2 {
            X(qAlice);
        }
    }

    operation DecodeMessageFromQubits_Reference(qAlice : Qubit, qBob : Qubit) : (Bool, Bool) {
        CNOT(qAlice, qBob);
        H(qAlice);
        return (MResetZ(qAlice) == One, MResetZ(qBob) == One);
    }


    // ------------------------------------------------------
    // Helper operation that runs superdense coding protocol using two building blocks
    // specified as first two parameters.
    operation ComposeProtocol(
        encodeOp : ((Qubit, (Bool, Bool)) => Unit),
        decodeOp : ((Qubit, Qubit) => (Bool, Bool)),
        message : (Bool, Bool)
    ) : (Bool, Bool) {

        use (qAlice, qBob) = (Qubit(), Qubit());

        CreateEntangledPairWrapper_Reference([qAlice, qBob]);

        encodeOp(qAlice, message);

        let (bit1, bit2) = decodeOp(qAlice, qBob);

        ResetAll([qAlice, qBob]);

        return (bit1, bit2);
    }

    // ------------------------------------------------------
    // Helper operation that runs superdense coding protocol (specified by protocolOp)
    // on all possible input values and verifies that decoding result matches the inputs
    operation CheckProtocolWithFeedback(protocolOp : ((Bool, Bool) => (Bool, Bool))) : Bool {

        // Loop over the 4 possible combinations of two bits
        for n in 0..3 {
            let data = (1 == n / 2, 1 == n % 2);
            let (dataBit1, dataBit2) = data;

            for iter in 1..100 {
                let (bit1, bit2) = protocolOp(data);

                // Now test if the bits were transfered correctly.
                if not (bit1 == dataBit1 and bit2 == dataBit2) {
                    Message("Incorrect.");
                    Message($"({dataBit1}, {dataBit2}) was transfered incorrectly as ({bit1}, {bit2})");
                    return false;
                }
            }
        }

        Message("Correct!");
        return true;
    }

}
