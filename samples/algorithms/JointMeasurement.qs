/// # Sample
/// Joint Measurement
///
/// # Description
/// Joint measurements, also known as Pauli measurements, are a generalization
/// of 2-outcome measurements to multiple qubits and other bases.
namespace Sample {
    open Microsoft.Quantum.Diagnostics;
    @EntryPoint()
        operation Main() : (Result, Result[]) {
        // Prepare an entangled state.
        use qs = Qubit[2];  // |00〉
        H(qs[0]);           // 1/sqrt(2)(|00〉 + |10〉)
        CNOT(qs[0], qs[1]); // 1/sqrt(2)(|00〉 + |11〉)

        // Show the quantum state before performing the joint measurement.
        DumpMachine();

        // The below code uses a joint measurement as a way to check the parity
        // of the first two qubits. In this case, the parity measurement result
        // will always be `Zero`.
        // Notice how the state was not collapsed by the joint measurement.
        let parityResult = Measure([PauliZ, PauliZ], qs[...1]); 
        DumpMachine();

        // However, if we perform a measurement just on the first qubit, we can
        // see how the state collapses.
        let firstQubitResult = M(qs[0]);
        DumpMachine();

        // Measuring the last qubit does not change the quantum state
        // since the state of the second qubit collapsed when the first qubit 
        // was measured because they were entangled.
        let secondQubitResult = M(qs[1]);
        DumpMachine();

        ResetAll(qs);
        return (parityResult, [firstQubitResult, secondQubitResult]);
    }
}