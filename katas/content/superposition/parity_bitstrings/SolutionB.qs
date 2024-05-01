namespace Kata {
    operation AllStatesWithParitySuperposition (qs : Qubit[], parity : Int) : Unit {
        use aux = Qubit();
        ApplyToEach(H, qs);
        ApplyToEach(CNOT(_, aux), qs);
        let res = MResetZ(aux);
        if ((res == Zero ? 0 | 1) != parity) {
            X(qs[0]);
        }
    }
}
