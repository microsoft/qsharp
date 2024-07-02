namespace Test {

    open Microsoft.Quantum.Intrinsic;

    @EntryPoint()
    operation Main() : Result {
        use q = Qubit();
        H(q);
        let r = M(q);
        Reset(q);
        return r;
    }
}
