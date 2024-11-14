namespace Sample {
    operation Main() : Result {
        use q = Qubit();
        X(q);
        Std.Diagnostics.DumpMachine();
        let r = M(q);
        Message($"The result of the measurement is {r}");
        Reset(q);
        r
    }
}
