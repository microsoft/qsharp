/// # Sample
/// Result
///
/// # Description
/// Represents the result of a projective measurement onto the eigenspaces
/// of a quantum operator with eigenvalues ±1. Possible values are `Zero` or `One`.
namespace MyQuantumApp {
    @EntryPoint()
    operation Main() : Result {
        // A `Result` can be declared with `Result` literals:
        let resultOne = One;
        let resultZero = Zero;

        // Or, it can be returned as the result of some quantum measurement, as
        // seen below:

        // Allocate a qubit.
        use q = Qubit();

        // Put qubit in superposition of |0> and |1> by applying the H gate to it.
        H(q);

        // Measure the qubit.
        let measurement = M(q);
        
        // Reset the qubit.
        Reset(q);

        // Return the measurement.
        return measurement;
    }
}