/// # Sample
/// Random Bit
///
/// # Description
/// This Q# program generates a random bit by setting a qubit in a superposition
/// of the computational basis states |0〉 and |1〉, and returning the measurement
/// result.
namespace Sample {

    @EntryPoint()
    operation RandomBit() : Result {
        // Qubits are only accesible for the duration of the scope where they
        // are allocated and are automatically released at the end of the scope.
        use qubit = Qubit();

        // Set the qubit in superposition by applying a Hadamard transformation.
        H(qubit);

        // Measure the qubit. There is a 50% probability of measuring either 
        // `Zero` or `One`.
        let result = M(qubit);

        // Reset the qubit so it can be safely released.
        Reset(qubit);
        return result;
    }
}
