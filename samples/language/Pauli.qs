/// # Sample
/// Pauli
///
/// # Description
/// Represents a single-qubit Pauli matrix. Possible values are `PauliI`, `PauliX`, `PauliY`, or `PauliZ`.
namespace MyQuantumApp {

    @EntryPoint()
    operation Main() : Result {
        use q = Qubit();
        
        // A `Pauli` can be declared as a literal.
        let pauliDimension = PauliX;
        
        // Measuring along a dimension returns a `Result`:
        let result = Measure([pauliDimension], [q]);

        return result;
    }
}