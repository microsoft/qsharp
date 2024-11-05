namespace Kata {
    operation TwoQubitState(qs : Qubit[]) : Int {
        // Implement your solution here...

        return -1;
    }

    // You might find this helper operation from an earlier task useful.
    operation BasisStateMeasurement(qs : Qubit[]) : Int {
        let m1 = M(qs[0]) == Zero ? 0 | 1;
        let m2 = M(qs[1]) == Zero ? 0 | 1;

        return m1 * 2 + m2;
    }
}
