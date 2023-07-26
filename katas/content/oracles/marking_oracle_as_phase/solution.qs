namespace Kata {

    // Task 2.1.
    operation ApplyMarkingOracleAsPhaseOracle(
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

}
