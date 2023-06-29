open Microsoft.Quantum.Diagnostics;

operation MultiQubitSystemsDemo () : Unit {
    // This allocates an array of 2 qubits, each of them in state |0⟩.
    // The overall state of the system is |00⟩
    use qs = Qubit[2];
    // X gate changes the first qubit into state |1⟩
    // The entire system is now in state |10⟩
    X(qs[0]);

    Message("System in state |10⟩:");
    DumpMachine();

    // This changes the second qubit into state |+⟩ = (1/sqrt(2))(|0⟩ + |1⟩).
    // The entire system is now in state (1/sqrt(2))(|10⟩ + |11⟩)
    H(qs[1]);

    Message("System in state (1/sqrt(2))(|10⟩ + |11⟩):");
    DumpMachine();

    // This changes the first qubit into state |-⟩ = (1/sqrt(2))(|0⟩ - |1⟩)
    // The entire system is now in state 0.5(|00⟩ + |01⟩ - |10⟩ - |11⟩)
    H(qs[0]);

    Message("System in state 0.5(|00⟩ + |01⟩ - |10⟩ - |11⟩):");
    DumpMachine();

    // You can use DumpRegister to examine the state of specific qubits rather than the entire simulator.
    // This prints the state of the first qubit
    Message("First qubit (in state |-⟩ = (1/sqrt(2))(|0⟩ - |1⟩):");
    DumpRegister((), [qs[0]]);

    // The next lines entangle the qubits.
    // Don't worry about what exactly they do for now
    H(qs[1]);
    CNOT(qs[0], qs[1]);

    Message("Entangled state 0.5(|00⟩ - |11⟩):");
    DumpMachine();

    // Since the states of entangled qubits are inseparable,
    // it makes no sense to examine only one of them
    Message("Let's try to examine one of two entangled qubits on its own...");
    DumpRegister((), [qs[0]]);

    // This returns the system into state |00⟩
    ResetAll(qs);
}
