namespace Sample {
    open Microsoft.Quantum.Diagnostics;

    operation main() : Result {
        use q1 = Qubit();
        use q2 = Qubit();
        Message("hi");

        H(q1);
        CNOT(q1, q2);
        DumpMachine();

        let m1 = M(q1);
        let m2 = M(q2);

        return [m1, m2];
    }
}
