/// # Sample
/// Joint Measurement
///
/// # Description
/// Joint measurements, also known as Pauli measurements, are a generalization
/// of 2-outcome measurements to multiple qubits and other bases.
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