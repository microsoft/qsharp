namespace Kata {
    operation TwoQubitStateTwo(qs : Qubit[]) : Int {
        H(qs[1]);

        CNOT(qs[0], qs[1]);
        H(qs[0]);

        let m1 = M(qs[0]) == One ? 0 | 1;
        let m2 = M(qs[1]) == One ? 0 | 1;

        return m2 * 2 + m1;
    }
}
