/// # Summary
/// Simple quantum teleportation sample
///
/// # Description
/// This Q# program demonstrates how to teleport quantum state
/// by communicating two classical bits using previously entangled qubits.
/// This code teleports |1〉 state, but any state can be teleported.
operation Main() : Bool {
    // Allocate qAlice, qBob qubits
    use (qAlice, qBob) = (Qubit(), Qubit());

    // Entangle qAlice, qBob qubits
    H(qAlice);
    CNOT(qAlice, qBob);

    // Allocate qMessage qubit and prepare it to be |1〉
    use qMessage = Qubit();
    X(qMessage);

    // Prepare the message by entangling with qAlice state
    CNOT(qMessage, qAlice);
    H(qMessage);

    // Obtain classical measurement results b1 and b2
    let b1 = M(qMessage) == One;
    let b2 = M(qAlice) == One;

    // Here classical results b1 and b2 are "sent" to the Bob's site.

    // Decode the message by applying adjustments based on classical data b1 and b2.
    if b1 {
        Z(qBob);
    }
    if b2 {
        X(qBob);
    }

    // Make sure that the obtained result is the same
    X(qBob);
    let correct = Std.Diagnostics.CheckZero(qBob);
    if correct {
        Message("Correct state!");
    } else {
        Message("Incorrect state!");
    }

    ResetAll([qAlice, qBob, qMessage]);

    // Return indication if the measurement of the state was correct
    correct
}
