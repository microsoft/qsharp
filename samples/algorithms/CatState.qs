/// # Sample
/// Cat State
///
/// # Description
/// A cat state is a highly entangled state where the qubits are in a
/// superposition of all |0...0〉 or all |0...0〉.
///
/// This Q# program implements a cat state.
namespace Sample {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Measurement;

    @EntryPoint()
    operation Main() : Result[] {
        use register = Qubit[5];

        // Prepare a cat state using the allocated register.
        PrepareCatState(register);

        // Show the cat state.
        DumpMachine();

        // Measure the and reset qubits before releasing them.
        let results = MeasureEachZ(register);
        ResetAll(register);
        return results;
    }

    // Prepares a cat state 1/sqrt(2)(|0...0〉 + |1...1〉) using a qubit register
    // in the zero state.
    operation PrepareCatState(register : Qubit[]) : Unit {
        Fact(Length(register) > 0, "Qubit register must not be empty.");
        Fact(
            CheckAllZero(register),
            "Qubits in register are not in the |0〉 state.");

        H(register[0]);
        for qubit in register[1...] {
            CNOT(register[0], qubit);
        }
    }
}
