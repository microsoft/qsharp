namespace Kata {
    operation MeasurementFreeTeleport(qAlice : Qubit, qBob : Qubit, qMessage : Qubit) : Unit {
        // The first part of the circuit is similar to Alice's part, but without measurements.
        CNOT(qMessage, qAlice);
        H(qMessage);
        
        // Classically controlled gates applied by Bob are replaced by controlled gates
        Controlled Z([qMessage], qBob);
        Controlled X([qAlice], qBob);
    }
}