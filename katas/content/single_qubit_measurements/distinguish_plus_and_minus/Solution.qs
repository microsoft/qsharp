namespace Kata {
    operation IsQubitMinus(q : Qubit) : Bool {
        return Measure([PauliX], [q]) == One;
    }
}
