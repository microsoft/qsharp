namespace Kata {
    operation ApplyMarkingOracleAsPhaseOracle(
        markingOracle : ((Qubit[], Qubit) => Unit is Adj + Ctl),
        qubits : Qubit[])
    : Unit is Adj + Ctl {
        // ...
    }
}
