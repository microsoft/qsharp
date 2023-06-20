namespace Kata.Reference {

    operation IsPlusPlusMinus (qs : Qubit[]) : Int {
        return Measure([PauliX], [qs[0]]) == Zero ? 0 | 1;
    }

}
