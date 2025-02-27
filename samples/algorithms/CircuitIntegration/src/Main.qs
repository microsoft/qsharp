/// # Sample
/// Circuit Integration
///
/// # Description
/// This sample demonstrates the ability to use circuit files in Q# projects.
/// It shows how to import and use custom quantum circuits defined in their own files.
/// In this sample, we import a circuit for performing a joint measurement of three
/// qubits, with one auxiliary qubit.

import JointMeasurement.JointMeasurement;

/// Sample program using custom gates from a hardware provider.
operation Main() : Result[] {
    use qs = Qubit[4];
    ApplyToEach(H, qs[0..3]);
    let results = JointMeasurement(qs[0], qs[1], qs[2], qs[3]);
    ResetAll(qs);
    results
}
