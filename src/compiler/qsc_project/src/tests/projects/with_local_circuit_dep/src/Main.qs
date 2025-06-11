operation Main() : Unit {
    use qs = Qubit[2];
    MyDep.Circuit1(qs);
    MyDep.Circuit2(qs);
}
