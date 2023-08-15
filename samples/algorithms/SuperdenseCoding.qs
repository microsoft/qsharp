/// # Sample
/// Superdense Coding
///
/// # Description
/// Superdense coding uses quantum entanglement to impact the state of 
/// entangled pairs of qubits while acting on only one of them.
/// Using this technique, one can encode two bits of information while
/// only touching one qubit. This is possible because of the shared
/// state of the two qubits.
namespace Sample {
    /// Entangles `qubit1` and `qubit2`, effecting an entangled state.
    operation CreateEntangledPair(qubit1 : Qubit, qubit2 : Qubit) : Unit {
      H(qubit1);
      // this `CNOT` gate will invert `qubit2` if the state of `qubit1` is
      // |1...
      CNOT(qubit1, qubit2);
    }

    @EntryPoint()
    operation Main(): Unit {
        use qs = Qubit[2];
        CreateEntangledPair(qs[1], qs[2]);

        // The below measurement has a 50% chance of being either 00 or 11
        Measure([PauliZ, PauliZ], qs) == One;
    }
}