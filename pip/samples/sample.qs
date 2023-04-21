operation AllBasisVectorsWithPhases_TwoQubits() : Unit {
    use q1 = Qubit();
    use q2 = Qubit();

    H(q1);
    Z(q1);

    H(q2);
    S(q2);

    Microsoft.Quantum.Diagnostics.DumpMachine();
}
