/// # Sample
/// Operations
///
/// # Description
/// Operations are the basic building blocks of a Q# program. A Q#
/// operation is a quantum subroutine. That is, it's a callable routine
/// that contains quantum operations that modify the state of qubits.
namespace MyQuantumApp {

    @EntryPoint()
    operation MeasureOneQubit() : Result {
        // Allocate a qubit, by default it is in zero state
        use q = Qubit();
        // We apply a Hadamard operation H to the state
        // It now has a 50% chance of being measured 0 or 1
        H(q);
        // Now we measure the qubit in Z-basis using the `M` operation.
        let result = M(q);
        Message($"Measurement result: {result}");
        // We reset the qubit before releasing it using the `Reset` operation.
        Reset(q);
        // Finally, we return the result of the measurement.
        return result;
    }
}