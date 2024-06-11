namespace Kata {
    operation MeasurementFreeTeleport(qAlice : Qubit, qBob : Qubit, qMessage : Qubit) : Unit {
        CNOT(qMessage, qAlice);
        H(qMessage);
        
        Controlled Z([qMessage], qBob);
        Controlled X([qAlice], qBob);
    }
}