namespace Kata {

    // Exercise 3.
    operation IsQubitMinus(q : Qubit): Bool {
        return Measure([PauliX], [q]) == One;
    }

}
