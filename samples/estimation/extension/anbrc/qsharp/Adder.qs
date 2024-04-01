namespace Samples {
    open Microsoft.Quantum.Unstable.Arithmetic;

    @EntryPoint()
    operation EstimateAdder() : Unit {
        let bitsize = 128;

        use xs = Qubit[bitsize];
        use ys = Qubit[bitsize];

        RippleCarryCGIncByLE(xs, ys);
    }
}
