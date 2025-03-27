import Std.Diagnostics.*;

/// # Summary
/// Greenberger–Horne–Zeilinger and Cat state sample
///
/// # Description
/// This Q# program shows how to prepare the GHZ state in a register of 3 qubits
/// and a generalized GHZ state (Cat state) in a register of 5 qubits.
///
/// # Remarks
/// The Greenberger–Horne–Zeilinger state, or GHZ state, is a state with 3
/// qubits defined as: |GHZ〉 = (|000〉 + |111〉) / √2.
///
/// The GHZ state is said to be a maximally entangled state, a multi-qubit
/// state where the state of any one qubit is not separable from the state
/// of any of the other qubits.
///
/// The generalized form of the GHZ state across any number of qubits is
/// called a Cat state, and the GHZ state is a special case of the Cat
/// state where the number of qubits is 3.
///
/// # References
/// - [Greenberger–Horne–Zeilinger state](https://en.wikipedia.org/wiki/Greenberger%E2%80%93Horne%E2%80%93Zeilinger_state)
operation Main() : (Result[], Result[]) {
    // Allocate 3 qubits for GHZ state.
    use ghz = Qubit[3];
    // Prepare a GHZ state in the register.
    PrepareGHZState(ghz);
    // Show the GHZ state in the register.
    DumpRegister(ghz);
    // Measure and reset qubits.
    let ghzResults = MResetEachZ(ghz);

    // Allocate 5 qubits for GHZ state.
    use cat5 = Qubit[5];
    // Prepare a Cat₅ state in the register.
    PrepareCatState(cat5);
    // Show the Cat₅ state in the register.
    DumpRegister(cat5);
    // Measure and reset qubits.
    let catResults = MResetEachZ(cat5);

    (ghzResults, catResults)
}

/// # Summary
/// Prepares state (|000〉 + |111〉) / √2 (GHZ state) in `qs` register.
/// All qubits are assumed to be in |0〉 state on input.
operation PrepareGHZState(qs : Qubit[]) : Unit {
    Fact(Length(qs) == 3, "Qubit register `qs` must be 3 qubits long.");

    // Set the first qubit into a (|0〉 + |1〉) / √2 superposition.
    H(qs[0]);
    // Entangle the first qubit with the second.
    CNOT(qs[0], qs[1]);
    // Entangle the second qubit with the third.
    CNOT(qs[1], qs[2]);
}

/// # Summary
/// Prepares Cat state (|0...0〉 + |1...1〉) / √2 (GHZ state) in `qs` register.
/// All qubits are assumed to be in |0〉 state on input.
operation PrepareCatState(qs : Qubit[]) : Unit {
    Fact(Length(qs) > 0, "Qubit register must not be empty.");

    // Set the first qubit into a (|0〉 + |1〉) / √2 superposition.
    H(qs[0]);

    // Then apply a CNOT to the remaining qubits using the first qubit as control.
    // This entangles first qubit with all the other qubits in the register.
    for q in qs[1...] {
        CNOT(qs[0], q);
    }
}
