/// # Sample
/// Majorana Qubits
///
/// # Description
/// In hardware providing majorana qubits, common quantum operations
/// are implemented using measurements and Pauli corrections. This
/// sample shows a hypotetical hardware provider exposing some custom
/// gates to Q# and a small library built on top of it.

/// Sample program using gates for custom hardware provider.
operation Main() : (Result, Result) {
    use qs = Qubit[2];
    GateSet.BellPair(qs[0], qs[1]);
    let res = GateSet.BellMeasurement(qs[0], qs[1]);
    ResetAll(qs);
    res
}
