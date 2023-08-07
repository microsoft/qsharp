namespace Kata {
    
    operation BasisStateMeasurement(qs: Qubit[]): Int {
        // Measurement on the first qubit gives the higher bit of the answer, on the second - the lower
        let m1 = M(qs[0]) == Zero ? 0 | 1;
        let m2 = M(qs[1]) == Zero ? 0 | 1;
        return m1 * 2 + m2;
    }
    
}
