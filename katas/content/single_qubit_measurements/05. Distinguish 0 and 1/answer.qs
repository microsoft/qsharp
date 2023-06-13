namespace Kata.Reference {
    // Exercise 2.
    operation IsQubitZero_Reference (q : Qubit) : Bool {
        return M(q) == Zero;
    }
}
