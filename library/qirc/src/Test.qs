import Qirc.Circuit.Circuit;
@EntryPoint()
operation QuantumTeleportation(): Unit {
    //  circuit = cirq.Circuit()
    let circ = new Circuit { operations = [] };

    //  Get the three qubits involved in the teleportation protocol.
    let msg =  Qirc.CreateNamedQubit("msg");
    let alice = Qirc.CreateNamedQubit("alice");
    let bob = Qirc.CreateNamedQubit("bob");
    M(msg.qubit);

    // Create a Bell state shared between Alice and Bob.
    let circ = Qirc.Circuit.Append([Gates.H(alice), Gates.CNOT(alice, bob)], circ);


    // Bell measurement of the Message and Alice's entangled qubit.
    let circ = Qirc.Circuit.Append([Gates.CNOT(msg, alice), Gates.H(msg), Gates.Measure(msg, alice)], circ);

    // Uses the two classical bits from the Bell measurement to recover the
    // original quantum message on Bob's entangled qubit.
    let circ = Qirc.Circuit.Append([Gates.CNOT(alice, bob), Gates.CZ(msg, bob)], circ);

    Qirc.Circuit.Execute(circ);
}