namespace Kata.Verification {

    // ------------------------------------------------------
    @EntryPoint()
    operation T41_ArbitraryBitPattern_Oracle () : Unit {
        for N in 1..4 {
            for k in 0..((2^N)-1) {
                let pattern = IntAsBoolArray(k, N);

                AssertTwoOraclesAreEqual(N..N, ArbitraryBitPattern_Oracle(_, _, pattern),
                                        ArbitraryBitPattern_Oracle_Reference(_, _, pattern));
            }
        }
    }

}
