namespace Kata.Verification {

    function Oracle_Converter (markingOracle : ((Qubit[], Qubit) => Unit is Adj + Ctl)) : (Qubit[] => Unit is Adj + Ctl) {
        return ApplyMarkingOracleAsPhaseOracle(markingOracle, _);
    }


    // ------------------------------------------------------
    @EntryPoint()
    operation T21_ApplyMarkingOracleAsPhaseOracle () : Unit {
        for N in 1..5 {
            for k in 0..(2^N-1) {
                let pattern = IntAsBoolArray(k, N);

                AssertOperationsEqualReferenced(N,
                                                Oracle_Converter(ArbitraryBitPattern_Oracle_Reference(_, _, pattern)),
                                                Oracle_Converter_Reference(ArbitraryBitPattern_Oracle_Reference(_, _, pattern)));
            }
        }
    }

}
