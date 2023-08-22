/// # Sample
/// Joint Measurement
///
/// # Description
/// The `Measure` operation performs a Pauli measurement. It performs a
/// joint measurement of one or more qubits in the specified Pauli bases.
/// The basis array and qubit array must be of the same length.
namespace Sample {
    open Microsoft.Quantum.Measurement;
    @EntryPoint()
    operation Main () : Result {
        // In Q#, the `Measure` operation performs a joint measurement of one or
        // more qubits in the specified Pauli bases. It takes two arguments: an
        // array of single-qubit Pauli values indicating the tensor product
        // factors on each qubit, and a register of qubits to be measured.

        // The below code uses a joint measurement as a way to check the parity
        // of two qubits.
        use qs = Qubit[2];
        H(qs[0]);
        CNOT(qs[0], qs[1]);
        let result = Measure([PauliZ, PauliZ], qs);

        return result;
    }
}