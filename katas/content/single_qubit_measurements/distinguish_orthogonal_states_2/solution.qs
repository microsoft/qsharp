
operation IsQubitA (alpha: Double, q : Qubit) : Bool { 
    Rx(-2.0 * alpha, q);
    return M(q) == Zero;
}

