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
        // `Measure` takes two arguments: first, an array `bases` of Pauli values, and
        // an array `qubits` of qubits to measure.

        // The below code shows joint measurement of a register of qubits.
        use qs = Qubit[2];
        let result = Measure([PauliZ, PauliZ], qs);

        return result;
    }
}