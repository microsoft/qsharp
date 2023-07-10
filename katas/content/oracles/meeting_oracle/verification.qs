namespace Kata.Verification {

    // ------------------------------------------------------
    @EntryPoint()
    operation T43_Meeting_Oracle () : Unit {
        for N in 1..4 {
            use jasmine = Qubit[N];
            for k in 0..(2^N-1) {
                let binaryJasmine = IntAsBoolArray(k, N);

                within {
                    ApplyPauliFromBitString(PauliX, true, binaryJasmine, jasmine);
                } apply {
                    AssertTwoOraclesAreEqual(1..N, Meeting_Oracle(_, jasmine, _),
                                            Meeting_Oracle_Reference(_, jasmine, _));
                }
            }
        }
    }

}
