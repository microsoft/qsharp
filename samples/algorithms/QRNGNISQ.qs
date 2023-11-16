/// # Sample
/// Quantum Random Number Generator
///
/// # Description
/// This program implements a quantum ranndom number generator by setting qubits
/// in superposition and then using the measurement results as random bits.
namespace Sample {
    open Microsoft.Quantum.Measurement;
    open Microsoft.Quantum.Intrinsic;

    @EntryPoint()
    operation Main() : Result[] {
        // Generate 5-bit random number.
        let nBits = 5;
        return GenerateNRandomBits(nBits);
    }

    /// # Summary
    /// Generates N random bits.
    operation GenerateNRandomBits(nBits : Int) : Result[] {
        // Allocate N qubits.
        use register = Qubit[nBits];

        // Set the qubits into superposition of 0 and 1 using the Hadamard
        // operation `H`.
        for qubit in register {
            H(qubit);
        }

        // At this point each has 50% chance of being measured in the |0〉 state
        // and 50% chance of being measured in the |1〉 state.
        // Measure each qubit and reset them all so they can be safely
        // deallocated.
        let results = MeasureEachZ(register);
        ResetAll(register);
        return results;
    }
}
