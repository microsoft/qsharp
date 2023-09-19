namespace Kata {
    open Microsoft.Quantum.Diagnostics;

    @EntryPoint()
    operation MultiQubitDumpMachineDemo() : Unit {
        // This line allocates two qubits in state |00⟩.
        use qs = Qubit[2];
        Message("State |00⟩:");

        // This line prints out the state of the quantum system.
        DumpMachine();

        // X gate changes the second qubit into state |1⟩.
        // The entire system is now in state |01⟩, or, in little-endian notation, |2⟩.
        X(qs[1]);
        Message("State |01⟩:");
        DumpMachine();

        CNOT(qs[1], qs[0]);
        Rx(1.0, qs[0]);
        Ry(2.0, qs[1]);
        Rz(3.0, qs[1]);

        Message("Uneven superposition state:");
        DumpMachine();

        // This line returns the qubits to state |0⟩ before releasing them.
        ResetAll(qs);
    }
}
