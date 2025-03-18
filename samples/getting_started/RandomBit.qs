/// # Summary
/// Random Bit sample
///
/// # Description
/// This Q# program generates a random bit by setting a qubit in a superposition
/// of the computational basis states |0〉 and |1〉, and returning the measurement
/// result.
operation Main() : Result {
    // Allocate a qubit.
    use qubit = Qubit();

    // Set the qubit in superposition by applying a Hadamard transformation.
    H(qubit);

    // Measure then reset the qubit.
    // There is a 50% probability of measuring either `Zero` or `One`.
    let result = MResetZ(qubit);

    result
}
