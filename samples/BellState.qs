namespace Microsoft.Quantum.Samples.BellState {
    open Microsoft.Quantum.Diagnostics;

    @EntryPoint()
    operation Main() : Result[] {
        use q1 = Qubit();
        use q2 = Qubit();

        H(q1);
        CNOT(q1, q2);
        DumpMachine();

        let m1 = M(q1);
        let m2 = M(q2);

        Reset(q1);
        Reset(q2);

        return [m1, m2];
    }
}
