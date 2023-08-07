namespace Kata {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;

    @EntryPoint()
     operation SingleQubitDumpMachineDemo () : Unit {
        // This line allocates a qubit in state |0⟩.
        use q = Qubit();
        Message("State |0⟩:");

        // This line prints out the state of the quantum system.
        // Since only one qubit is allocated, only its state is printed.
        DumpMachine();

        // This line changes the qubit to state |+⟩ = (1/sqrt(2))(|0⟩ + |1⟩).
        // 1/sqrt(2) is approximately 0.707107.
        H(q);

        Message("State |+⟩:");
        DumpMachine();

        // This will put the qubit into an uneven superposition,
        // where the amplitudes of |0⟩ and |1⟩ have different absolute values and relative phases.
        Rx(1.0, q);
        Ry(2.0, q);
        Rz(3.0, q);

        Message("Uneven superposition state:");
        DumpMachine();

        // This line returns the qubit to state |0⟩ before releasing it.
        Reset(q);
    }
}
