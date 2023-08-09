namespace Kata.Verification {
    open Microsoft.Quantum.Convert;

    operation ApplyMarkingOracleAsPhaseOracle_Reference(
        markingOracle: ((Qubit[], Qubit) => Unit is Adj + Ctl),
        qubits: Qubit[]) : Unit is Adj + Ctl {
            
        use minus = Qubit();
        within {
            X(minus);
            H(minus);
        } apply {
            markingOracle(qubits, minus);
        }
    }

    @EntryPoint()
    operation CheckSolution(): Bool {
        for N in 1..5 {
            for k in 0..(2^N-1) {
                let pattern = IntAsBoolArray(k, N);

                let isCorrect = CheckOperationsEqualReferenced(
                    N,
                    qubits => Kata.ApplyMarkingOracleAsPhaseOracle(
                        ApplyControlledOnBitString(pattern, X, _, _),
                        qubits),
                    qubits => ApplyMarkingOracleAsPhaseOracle_Reference(
                        ApplyControlledOnBitString(pattern, X, _, _),
                        qubits));
                if not isCorrect {
                    Message($"Failed on test pattern {pattern} for a bit pattern oracle.");
                    return false;
                }
            }
        }
        Message("All tests passed.");
        true
    }

}
