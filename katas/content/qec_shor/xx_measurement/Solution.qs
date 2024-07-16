namespace Kata {
    operation XXMeasurement(qs : Qubit[]) : Int {
        return Measure([PauliX, PauliX], qs) == Zero ? 0 | 1;
    }
}
