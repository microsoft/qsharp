/// # Sample
/// Bell State
///
/// # Description
/// Bell states or EPR pairs are specific quantum states of two qubits
/// that represent the simplest (and maximal) examples of quantum entanglement.
/// The following Q# program implements one of these Bell states.
namespace Sample {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Diagnostics;

    @EntryPoint()
    operation BellState() : (Result, Result) {
        // Allocate the two qubits that will be used to create a Bell state.
        use q1 = Qubit();
        use q2 = Qubit();

        // Set the first qubit in superposition by calling the `H` operation, 
        // which applies a Hadamard transformation to the qubit.
        // Then, entangle the two qubits using the `CNOT` operation.
        H(q1);
        CNOT(q1, q2);

        // Show the state of the two qubits using the `DumpMachine` function.
        DumpMachine();

        // Measure the two qubits and reset them before they are released at the
        // end of the block.
        let m1 = M(q1);
        let m2 = M(q2);
        Reset(q1);
        Reset(q2);

        // Return the measurement results.
        return (m1, m2);
    }
}
