/// A set of gates built upon the custom measurements
/// provided by the hardware provider.
///
/// Source:
///  [1] Surface code compilation via edge-disjoint paths
///      https://arxiv.org/pdf/2110.11493

/// Apply a CNOT gate to the given qubits.
/// Source: [1] Figure 3.
operation CNOT(control : Qubit, target : Qubit) : Unit {
    use ancilla = Qubit();
    let a = Mzz(control, target);
    let b = Mxx(target, ancilla);
    let c = Mx(target);
    if b == One {
        Z(control);
    }
    if [a, c] == [One, One] {
        X(ancilla);
    }
}

/// Prepare a Bell Pair.
/// Source: [1] Figure 18a.
operation BellPair(q1 : Qubit, q2 : Qubit) : Unit {
    // Bring the qubits to their ground state.
    Mz(q1);
    Mz(q2);

    // Bell Pair preparation
    if Mxx(q1, q2) == One {
        Z(q2);
    }
}

/// Measure a Bell Pair.
/// Source: [1] Figure 18b.
operation BellMeasurement(q1 : Qubit, q2 : Qubit) : (Result, Result) {
    let z = Mzz(q1, q2);
    let x = Mxx(q1, q2);
    (x, z)
}

/// User friendly wrapper around the Mx hardware gate.
operation Mx(q : Qubit) : Result {
    HardwareIntrinsics.__quantum__qis__mx__body(q)
}

/// User friendly wrapper around the Mz hardware gate.
operation Mz(q : Qubit) : Result {
    HardwareIntrinsics.__quantum__qis__mz__body(q)
}

/// User friendly wrapper around the Mxx hardware gate.
operation Mxx(q1 : Qubit, q2 : Qubit) : Result {
    HardwareIntrinsics.__quantum__qis__mxx__body(q1, q2)
}

/// User friendly wrapper around the Mzz hardware gate.
operation Mzz(q1 : Qubit, q2 : Qubit) : Result {
    HardwareIntrinsics.__quantum__qis__mzz__body(q1, q2)
}
