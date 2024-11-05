namespace Kata {
    operation IsQubitA(alpha : Double, q : Qubit) : Bool {
        Ry(-2.0 * alpha, q);
        return M(q) == Zero;
    }
}
