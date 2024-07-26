// # Sample
// Pauli
//
// # Description
// Represents a single-qubit Pauli matrix. Possible values are `PauliI`, `PauliX`, `PauliY`, or `PauliZ`.

operation Main() : Result {
    use q = Qubit();

    // A `Pauli` can be declared as a literal.
    let pauliDimension = PauliX;
    Message($"Pauli dimension: {pauliDimension}");

    // Measuring along a dimension returns a `Result`:
    let result = Measure([pauliDimension], [q]);
    Message($"Measurement result: {result}");

    // Reset the qubit before releasing it to ensure it is in the |0⟩ state.
    Reset(q);

    return result;
}
