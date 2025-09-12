operation Main() : Unit {
    use qs = Qubit[2];

    Adjoint H(qs[0]);

    mutable arg = 2.0;
    if (M(qs[0]) == One) {
        set arg = 1.0;
    }
    Rz(arg, qs[1]);

    ResetAll(qs);
}
