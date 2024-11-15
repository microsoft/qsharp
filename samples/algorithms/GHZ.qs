/// # Sample
/// GHZ
///
/// # Description
/// The Greenberger–Horne–Zeilinger state, or GHZ state, is a state with 3
/// qubits defined as: |GHZ〉 = (|000〉 + |111〉) / √2.
///
/// The GHZ state is said to be a maximally entangled state, a multi-qubit
/// state where the state of any one qubit is not separable from the state
/// of any of the other qubits.
///
/// The generalized form of the GHZ state across any number of qubits is
/// called a cat state, and the GHZ state is a special case of the cat
/// state where the number of qubits is 3.
///
/// This Q# program prepares the GHZ state in a register of 3 qubits, then
/// returns the result of measuring those qubits.
import Std.Diagnostics.*;

operation Main() : Result[] {
    use qs = Qubit[3];

    // Prepare a GHZ state using the allocated register.
    PrepareGHZState(qs);

    // Show the GHZ state.
    DumpMachine();

    // Measure and reset qubits before releasing them.
    MResetEachZ(qs)
}

/// # Summary
/// Prepares state (|000〉 + |111〉) / √2 (GHZ state) across a register
/// of three qubits `qs`.
/// All qubits are assumed to be in |0〉 state on input.
operation PrepareGHZState(qs : Qubit[]) : Unit {
    Fact(Length(qs) == 3, "`qs` length shuold be 3.");

    H(qs[0]);
    CNOT(qs[0], qs[1]);
    CNOT(qs[0], qs[2]);
}
