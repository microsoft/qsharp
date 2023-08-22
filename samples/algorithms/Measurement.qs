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
    operation Main () : (Result, Result[], Result) {
        // The `M` operation performs a measurement of a single qubit in the
        // computational basis, also known as the Pauli Z basis.
        use q = Qubit();
        let result = M(q);
        Reset(q);

        // The `MeasureEachZ` operation measures each qubit in an array in the
        // computational basis and returns an array of `Result` values.
        use qs = Qubit[2];
        let results = MeasureEachZ(qs);

        // The `Measure` operation performs a joint measurement of one or more
        // qubits in the specified Pauli bases.
        H(qs[0]);
        CNOT(qs[0], qs[1]);
        let jointResult = Measure([PauliZ, PauliZ], qs);
        ResetAll(qs);

        return (result, results, jointResult);
    }
}