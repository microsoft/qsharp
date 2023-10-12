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
namespace Sample {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Measurement;

    @EntryPoint()
    operation Main() : Result[] {
        use qs = Qubit[3];

        // Prepare a GHZ state using the allocated register.
        PrepareGHZState(qs);

        // Show the GHZ state.
        DumpMachine();

        // Measure and reset qubits before releasing them.
        let results = MeasureEachZ(qs);
        ResetAll(qs);
        return results;
    }

    /// # Summary
    /// This operation prepares a generalized GHZ state across a register of qubits.
    ///
    /// # Input
    /// ## qs
    /// The given register of qubits to be transformed into the GHZ state. It is assumed
    /// that these qubits are in their default |0〉 state.
    ///
    /// # Output
    /// The operation returns `Unit`. Additionally, the given register of qubits will
    /// be in the GHZ state.
    operation PrepareGHZState(qs : Qubit[]) : Unit {
        if Length(qs) > 0 {
            H(qs[0]);
        }

        for q in qs[1...] {
            CNOT(qs[0], q);
        }
    }
}
