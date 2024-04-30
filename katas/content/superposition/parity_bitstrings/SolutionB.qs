namespace Kata {
    open Microsoft.Quantum.Measurement;

    operation AllStatesWithParitySuperposition (qs : Qubit[], parity : Int) : Unit {
        use aux = Qubit();
        ApplyToEach(H, qs);
        ApplyToEach(CNOT(_, aux), qs);
        let res = M(aux);
        if ((res == Zero ? 0 | 1) != parity) {
            X(qs[0]);
        }

        Reset(aux);
    }
}
