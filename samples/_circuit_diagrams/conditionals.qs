operation Main() : Unit {
    use qs = Qubit[2];

    H(qs[0]);

    if (M(qs[0]) == One) {
        X(qs[1]);
    }
    ResetAll(qs)

}
