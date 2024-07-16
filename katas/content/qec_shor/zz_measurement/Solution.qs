namespace Kata {
    operation ZZMeasurement(qs : Qubit[]) : Int {
        return Measure([PauliZ, PauliZ], qs) == Zero ? 0 | 1;
    }
}
