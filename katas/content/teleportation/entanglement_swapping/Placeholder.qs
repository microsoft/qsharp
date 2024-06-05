namespace Kata {
    operation EntanglementSwapping() : ((Qubit, Qubit) => (Bool, Bool), (Qubit, (Bool, Bool)) => Unit) {
        return (SendMessage, ReconstructMessage);
    }

    // You might find these helper operations from earlier tasks useful.
    operation SendMessage(qAlice: Qubit, qMessage: Qubit) : (Bool, Bool) {
        CNOT(qMessage, qAlice);
        H(qMessage);
        return (M(qMessage) == One, M(qAlice) == One);
    }

    operation ReconstructMessage(qBob : Qubit, (b1 : Bool, b2 : Bool)) : Unit {
        if b1 {
            Z(qBob);
        }
        if b2 {
            X(qBob);
        }
    }
}