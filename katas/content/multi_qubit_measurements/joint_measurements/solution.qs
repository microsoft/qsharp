namespace Kata.Reference {

    operation ParityMeasurement (qs : Qubit[]) : Int {
        return Measure([PauliZ, PauliZ], qs) == Zero ? 0 | 1;
    }
    
}
