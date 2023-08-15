/// # Sample
/// Measurement
///
/// # Description
/// Measurement is a critical component of quantum algorithm design. Measurement refers to
/// measurements in the computational basis, unless otherwise specified. The result of
/// a measurement is a value of the type `Result`, that is, `One` or `Zero`. This appears
/// similar to classical computation, but the measurement itself is a probabilistic process.
/// The process of measuring a qubit can change the state of the qubit.
namespace Sample {
    open Microsoft.Quantum.Measurement;
    @EntryPoint()
    operation Main () : Result {
        // There are two main measurement operations in Q#.
        // The first is `M`, which measures a single qubit in the computational basis.
        // The below line allocates a qubit. After that, we will measure it with `M`.
        use q = Qubit();
        let result: Result = M(q);

        // The second is `Measure`, a more explicit variant that performs joint measurement.
        // When using `Measure`, you must specify the `base` Pauli values indicating the
        // tensor product factors on each qubit. The second argument is the register of
        // qubits to be measured.  Note that the below call to `Measure` is equivalent
        // to the above call to `M`.

        use q = Qubit();
        let result: Result = Measure([PauliZ], [q]);

        // The below code shows joint measurement of a register of qubits.
        use qs = Qubit[2];
        let result = Measure([PauliZ, PauliZ], qs);

        return result;
    }
}