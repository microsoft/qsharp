/// # Sample
/// Measurement
///
/// # Description
/// Quantum measurement is an irreversible operation in which a quantum system
/// is manipulated to yield a numerical result. Measuring a quantum system
/// generally changes the quantum state that describes that system.
///
/// In Q#, the result of a measurement is a value of the type `Result`, that is,
/// `One` or `Zero`.
///
/// This Q# program exemplifies different types of measurements.
namespace Sample {
    open Microsoft.Quantum.Measurement;
    @EntryPoint()
    operation Main () : (Result, Result[]) {
        // The `M` operation performs a measurement of a single qubit in the
        // computational basis, also known as the Pauli Z basis.
        use q = Qubit();
        let result = M(q);
        Reset(q);

        // The `MeasureEachZ` operation measures each qubit in an array in the
        // computational basis and returns an array of `Result` values.
        use qs = Qubit[2];
        let results = MeasureEachZ(qs);

        return (result, results);
    }
}