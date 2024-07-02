namespace Kata {
    operation BellState(qs : Qubit[]) : Int {
        CNOT(qs[0], qs[1]);
        H(qs[0]);

        let m1 = M(qs[0]) == Zero ? 0 | 1;
        let m2 = M(qs[1]) == Zero ? 0 | 1;

        return m2 * 2 + m1;
    }
}
