namespace Kata {
    operation IsQubitPlus(q : Qubit) : Bool {
        return Measure([PauliX], [q]) == Zero;
    }
}
