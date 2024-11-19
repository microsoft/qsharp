/// # Sample
/// Majorana Qubits
///
/// # Description
/// In hardware providing majorana qubits, common quantum operations
/// are implemented using measurements and Pauli corrections. This
/// sample shows a hypotetical hardware provider exposing some custom
/// gates to Q# and a small library built on top of it.

/// Sample program using custom gates from a hardware provider.
operation Main() : (Result, Result) {
    use qs = Qubit[2];
    GateSet.BellPair(qs[0], qs[1]);

    // Applying X to any of the qubits will result in the (Zero, One) Bell state.
    // X(qs[0]); // Uncomment to try

    // Applying Z to any of the qubits will result in the (One, Zero) Bell state.
    // Z(qs[0]); // Uncomment to try

    // Applying X and Z to the pair will result in the (One, One) Bell state.
    // Note they can be applied to the same Qubit.
    // Z(qs[0]); // Uncomment to try
    // X(qs[0]);

    let res = GateSet.BellMeasurement(qs[0], qs[1]);
    ResetAll(qs);
    res
}
