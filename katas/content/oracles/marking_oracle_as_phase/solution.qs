namespace Quantum.Kata.Reference {

    // Task 2.1.
    operation ApplyMarkingOracleAsPhaseOracle_Reference (markingOracle : ((Qubit[], Qubit) => Unit is Adj + Ctl), qubits : Qubit[]) : Unit is Adj + Ctl {
        use minus = Qubit();
        within {
            X(minus);
            H(minus);
        } apply {
            markingOracle(qubits, minus);
        }
    }

}
