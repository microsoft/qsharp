namespace Kata.Verification {

    // ------------------------------------------------------
    @EntryPoint()
    operation T42_ArbitraryBitPattern_Oracle_Challenge () : Unit {
        for N in 1..4 {
            for k in 0..((2^N)-1) {
                let pattern = IntAsBoolArray(k, N);

                within {
                    AllowAtMostNQubits(2*N, "You are not allowed to allocate extra qubits");
                } apply {
                    AssertOperationsEqualReferenced(N,
                                                    ArbitraryBitPattern_Oracle_Challenge(_, pattern),
                                                    ArbitraryBitPattern_Oracle_Challenge_Reference(_, pattern));
                }
            }
        }
    }

}
