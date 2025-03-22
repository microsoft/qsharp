/// # Summary
/// Measurement sample
///
/// # Description
/// This Q# program demonstrates how to perform measurements in Z basis.
///
/// # Remarks
/// Quantum measurement is an irreversible operation in which a quantum system
/// is manipulated to yield a numerical result. Measuring a quantum system
/// generally changes the quantum state that describes that system.
operation Main() : (Result, Result[]) {
    use q = Qubit(); // Allocate a qubit. Qubit is in |0〉 state after allocation.
    X(q); // Flip the state. Qubit is in |1〉 state now.

    // The `M` operation performs a measurement of a single qubit in the
    // computational basis, also known as the Pauli Z basis.
    let result = M(q);

    // Reset qubit back to |0〉 state.
    // An alternative way to measure and then reset a qubit is to use
    // `MResetZ` operation.
    Reset(q);

    use qs = Qubit[2]; // Allocate a two-qubit array (or register).

    // The `MeasureEachZ` operation measures each qubit in an array in the
    // computational basis and returns an array of `Result` values.
    let results = MeasureEachZ(qs);

    // Return all results. In Q#, the result of a measurement is a value
    // of the type `Result`, that is, `One` or `Zero`.
    (result, results)
}
