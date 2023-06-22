namespace Example {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;

    @EntryPoint()
    operation MultipleQubits() : Unit {
        // This allocates an array of 2 qubits, each of them in state |0⟩.
        // The overall state of the system is |00⟩
        use register = Qubit[2];

        // We can use the `DumpMachine` function to show the state of the entire quantum system.
        Message("System in state |00⟩:");
        DumpMachine();

        // X gate changes the first qubit into state |1⟩
        // The entire system is now in state |10⟩
        X(register[0]);

        Message("System in state |10⟩:");
        DumpMachine();

        // This changes the second qubit into state |+⟩ = (1/sqrt(2))(|0⟩ + |1⟩).
        // The entire system is now in state (1/sqrt(2))(|10⟩ + |11⟩)
        H(register[1]);

        Message("System in state (1/sqrt(2))(|10⟩ + |11⟩):");
        DumpMachine();

        // This changes the first qubit into state |-⟩ = (1/sqrt(2))(|0⟩ - |1⟩)
        // The entire system is now in state 0.5(|00⟩ + |01⟩ - |10⟩ - |11⟩)
        H(register[0]);

        Message("System in state 0.5(|00⟩ + |01⟩ - |10⟩ - |11⟩):");
        DumpMachine();

        // The next lines entangle the qubits.
        // Don't worry about what exactly they do for now
        H(register[1]);
        CNOT(register[0], register[1]);

        Message("Entangled state 0.5(|00⟩ - |11⟩):");
        DumpMachine();

        // This returns the system into state |00⟩
        ResetAll(register);
    }
}