/// # Sample
/// Simple Quantum Phase Estimation
///
/// # Description
/// This sample demonstrates how to use ApplyQPE to perform
/// Quantum Phase Estimation of a simple unitary operation.
/// Rz gate is used as the unitary operation,
/// and the eigenvector |1⟩ is chosen for the phase estimation.
/// Run this sample and check the histogram of results.

/// # Summary
/// Estimate the phase of the eigenvalue of the unitary gate U
/// using Quantum Phase Estimation (QPE) algorithm.
/// The result is returned as an array of {0,1} integers
/// representing the binary fraction of the phase in big-endian order.
/// This is used to display the histogram of the results.
operation Main() : Int[] {
    // Allocate qubits to be used in phase estimation
    use state = Qubit();
    use phase = Qubit[6];

    // Prepare Rz eigenvector |1⟩
    X(state);

    // ApplyOperationPowerCA is used to apply the unitary operation U multiple times
    // as required by the Quantum Phase Estimation algorithm. In general, unitary U may
    // operate on multiple qubits, but in this case it operates on a single qubit qs[0].
    let oracle = ApplyOperationPowerCA(_, qs => U(qs[0]), _);

    // Estimate eigenvalue of Rz gate corresponding eigenvector |1⟩
    // In general, unitary U may operate on multiple qubits, but in this case
    // it operates on a single qubit state. So a single element array [state] is passed
    // as the target state.
    ApplyQPE(oracle, [state], phase);

    // Measure each qubit in the phase register
    let results = MeasureEachZ(phase);

    // Reset the qubits
    Reset(state);
    ResetAll(phase);

    // Convert result array to array of {0,1} integers for display purposes
    mutable fraction : Int[] = [];
    // Reverse the order of the results to present in big-endian
    for i in Length(results) - 1..-1..0 {
        fraction += [if results[i] == One { 1 } else { 0 }];
    }

    // For the gate Rz(π/3) and the eigenvector |1⟩, the eigenvalue should be exp(i * π/6).
    // Its phase π/6 is 1/12 as a frction of 2π, or ~0.0001010101 in binary.
    // The expected histogram should therefore peak at [0, 0, 0, 1, 0, 1] (big-endian order).
    // with the smaller bar at a bigger value [0, 0, 0, 1, 1, 0]

    // Return one data point for histogram display
    fraction
}


// U is the unitary gate for which we want to perform phase estimation.
operation U(q : Qubit) : Unit is Ctl + Adj {
    // Using the Rz gate with the angle of π/3
    // The eigenvalues of this gate are exp(-i * π/6) for |0⟩ and exp(i * π/6) for |1⟩.
    Rz(Std.Math.PI() / 3.0, q);
}

