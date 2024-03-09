namespace Kata {
    open Microsoft.Quantum.Diagnostics;

    @EntryPoint()
    operation MultiQubitSystemsDemo () : Unit {
        // This allocates an array of 2 qubits, each of them in state |0⟩.
        // The overall state of the system is |00⟩.
        use qs = Qubit[2];
        // X gate changes the first qubit into state |1⟩.
        X(qs[0]);
        Message("The system in now in state |10⟩:");
        DumpMachine();

        // This changes the second qubit into state |+⟩ = (1/sqrt(2))(|0⟩ + |1⟩).
        H(qs[1]);
        Message("The system in now in state  (1/sqrt(2))(|10⟩ + |11⟩):");
        DumpMachine();

        // This changes the first qubit into state |-⟩ = (1/sqrt(2))(|0⟩ - |1⟩)
        H(qs[0]);
        Message("The system in now in state 0.5(|00⟩ + |01⟩ - |10⟩ - |11⟩):");
        DumpMachine();

        // The next lines entangle the qubits (don't worry about what exactly they do for now).
        H(qs[1]);
        CNOT(qs[0], qs[1]);
        Message("The system in now in entangled state 0.5(|00⟩ - |11⟩):");
        DumpMachine();

        // This returns the system into state |00⟩.
        ResetAll(qs);
    }
}