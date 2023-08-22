/// # Sample
/// Superdense Coding
///
/// # Description
/// Superdense coding is a quantum communication protocol to communicate a
/// number of classical bits of information by only transmitting a smaller
/// number of qubits.
///
/// This Q# program implements superdense coding to send two classical bits of
/// information.
namespace Sample {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Random;

    @EntryPoint()
    operation Main(): (Bool, Bool) {
        use (aliceQubit, bobQubit) = (Qubit(), Qubit());

        // The protocol starts with the preparation of an entangled state, which
        // is later shared between Alice and Bob.
        CreateEntangledPair(aliceQubit, bobQubit);

        // Alice encodes 2 random bits in her qubit of the entangled pair.
        let randomInt = DrawRandomInt(0, 3);
        let bit1 = (randomInt &&& 1) == 0b01;
        let bit2 = (randomInt &&& 2) == 0b10;
        SuperdenseEncode(bit1, bit2, aliceQubit);

        // "Send" Alice's qubit to Bob and let Bob decode two bits.
        let (decodedBit1, decodedBit2) = SuperdenseDecode(aliceQubit, bobQubit);

        ResetAll([aliceQubit, bobQubit]);
        return (decodedBit1, decodedBit2);
    }

    // Prepares an entangled state: 1/sqrt(2)(|00〉 + |11〉)
    operation CreateEntangledPair(q1 : Qubit, q2 : Qubit) : Unit {
        Fact(
            CheckAllZero([q1, q2]),
            "Qubits are expected to be in the |00〉 state");

        H(q1);
        CNOT(q1, q2);
    }

    // Encodes two bits of information in one qubit. The qubit is expected to
    // be a half of an entangled pair.
    operation SuperdenseEncode(bit1 : Bool, bit2 : Bool, qubit : Qubit) : Unit {
        if (bit1) {
            Z(qubit);
        }
        if (bit2) {
            X(qubit);
        }
    }

    // Decodes two bits of information from a joint state of two qubits.
    operation SuperdenseDecode(qubit1 : Qubit, qubit2 : Qubit) : (Bool, Bool) {
        // If bit1 in the encoding procedure was true we applied Z to the first
        // qubit which anti-commutes with XX, therefore bit1 can be read out
        // from XX measurement.
        let bit1 = Measure([PauliX, PauliX], [qubit1, qubit2]) == One;

        // If bit2 in the encoding procedure was true we applied X to the first
        // qubit which anti-commutes with ZZ, therefore bit2 can be read out
        // from ZZ measurement.
        let bit2 = Measure([PauliZ, PauliZ], [qubit1, qubit2]) == One;
        return (bit1, bit2);
    }
}