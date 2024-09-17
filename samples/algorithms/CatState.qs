/// # Sample
/// Cat State
///
/// # Description
/// A cat state is a highly entangled state where the qubits are in a
/// superposition of all |0...0〉 or all |1...1〉.
///
/// This Q# program implements a cat state of 5 qubits.
namespace Sample {
    import Std.Diagnostics.*;

    @EntryPoint()
    operation Main() : Result[] {
        use register = Qubit[5];

        // Prepare a cat state using the allocated register.
        PrepareCatState(register);

        // Show the cat state.
        DumpMachine();

        // Measure and reset qubits before releasing them.
        MResetEachZ(register)
    }

    /// # Summary
    /// Prepares state (|0...0〉 + |1...1〉) / √2 (a generalized GHZ state
    /// or a cat state) across the `register` of qubits.
    /// All qubits are assumed to be in |0〉 state on input.
    operation PrepareCatState(register : Qubit[]) : Unit {
        Fact(Length(register) > 0, "Qubit register must not be empty.");

        // Set the first qubit in the register into a (|0〉 + |1〉) / √2 superposition.
        // Then apply a CNOT to the remaining qubits using the first qubit as control.
        H(register[0]);
        ApplyToEach(CNOT(register[0], _), register[1...]);
    }
}
