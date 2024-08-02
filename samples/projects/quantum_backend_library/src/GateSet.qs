/// # Sample
/// Quantum Backend Gateset
///
/// # Description
/// This Q# file contains the implementation of the native gateset for a
/// hypothetical quantum backend.
///
/// User-friendly named operations that represent the native gateset for this
/// quantum backend are exported. The exact native gates for this quantum
/// backend are implemented as `@SimulatableIntrinsic` operations so users
/// calling them in their programs can both use a simulator and generate code
/// that will execute in the quantum backend without making any modifications to
/// their programs.

/// # Summary
/// Native operation that applies a rotation about the _x_-axis by a given angle.
///
/// # Input
/// ## theta
/// Angle about which the qubit is to be rotated.
/// ## qubit
/// Qubit to which the gate should be applied.
operation Rx(theta : Double, q : Qubit) : Unit {
    __quantum__qis__rx__body(theta, q);
}

/// # Summary
/// Native operation that applies a rotation about the _z_-axis by a given angle.
///
/// # Input
/// ## theta
/// Angle about which the qubit is to be rotated.
/// ## qubit
/// Qubit to which the gate should be applied.
operation Rz(theta : Double, q : Qubit) : Unit {
    __quantum__qis__rz__body(theta, q);
}

/// # Summary
/// Native operation that applies the two qubit Ising _ZZ_ rotation gate.
///
/// # Input
/// ## theta
/// The angle about which the qubits are rotated.
/// ## qubit0
/// The first qubit input to the gate.
/// ## qubit1
/// The second qubit input to the gate.
operation Rzz(theta : Double, qubit0 : Qubit, qubit1 : Qubit) : Unit {
    __quantum__qis__rzz__body(theta, qubit0, qubit1);
}

/// # Summary
/// Native operation that measures a single qubit in the Z basis, and resets it
/// to a fixed initial state following the measurement.
///
/// # Input
/// ## target
/// A single qubit to be measured.
///
/// # Output
/// The result of measuring `target` in the Pauli Z basis.
operation MResetZ(q : Qubit) : Result {
    QIR.Intrinsic.__quantum__qis__mresetz__body(q)
}

/// # Summary
/// Native operation that ensures it is in the |0⟩ state.
///
/// # Input
/// ## qubit
/// The qubit whose state is to be reset to |0⟩.
operation Reset(q : Qubit) : Unit {
    QIR.Intrinsic.__quantum__qis__reset__body(q);
}

@SimulatableIntrinsic()
operation __quantum__qis__rx__body(theta : Double, q : Qubit) : Unit {
    Std.Intrinsic.Rx(theta, q);
}

@SimulatableIntrinsic()
operation __quantum__qis__rz__body(theta : Double, q : Qubit) : Unit {
    Std.Intrinsic.Rz(theta, q);
}

@SimulatableIntrinsic()
operation __quantum__qis__rzz__body(
    theta : Double,
    qubit0 : Qubit,
    qubit1 : Qubit
) : Unit {
    Std.Intrinsic.Rzz(theta, qubit0, qubit1);
}

export Rx, Rz, Rzz, MResetZ, Reset;
