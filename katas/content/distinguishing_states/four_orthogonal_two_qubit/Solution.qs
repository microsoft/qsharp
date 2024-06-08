namespace Kata {
    operation TwoQubitState(qs : Qubit[]) : Int {
        H(qs[0]);
        H(qs[1]);

        return BasisStateMeasurement(qs);
    }

    operation BasisStateMeasurement(qs : Qubit[]) : Int {
        let m1 = M(qs[0]) == Zero ? 0 | 1;
        let m2 = M(qs[1]) == Zero ? 0 | 1;

        return m1 * 2 + m2;
    }
}
