/// # Sample
/// Cat State
///
/// # Description
/// A cat state is a highly entangled state where the qubits are in a
/// superposition of all |0...0〉 or all |1...1〉.
///
/// This Q# program implements a cat state of 5 qubits.
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

        // Measure and reset qubits before releasing them.
        let results = MeasureEachZ(register);
        ResetAll(register);
        return results;
    }

    // Prepares a cat state 1/sqrt(2)(|0...0〉 + |1...1〉) using a qubit register
    // in the zero state.
    operation PrepareCatState(register : Qubit[]) : Unit {
        Fact(Length(register) > 0, "Qubit register must not be empty.");
        Fact(CheckAllZero(register), "Qubits are not in the |0〉 state.");

        // Set the first qubit in the register into a 1/sqrt(2)(|0〉 + |1〉) superposition.
        // Then apply a CNOT to the remaining qubits using the first qubit as control.
        H(register[0]);
        ApplyToEach(CNOT(register[0], _), register[1...]);
    }
}
