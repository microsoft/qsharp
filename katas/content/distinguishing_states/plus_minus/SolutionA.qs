namespace Kata {
    operation IsQubitPlus(q : Qubit) : Bool {
        H(q);
        return M(q) == Zero;
    }
}
