namespace Kata {
    operation StandardTeleport(qAlice : Qubit, qBob : Qubit, qMessage : Qubit) : Unit {
        Entangle(qAlice, qBob);
        let classicalBits = SendMessage(qAlice, qMessage);
        ReconstructMessage(qBob, classicalBits);
    }

    operation Entangle(qAlice : Qubit, qBob : Qubit) : Unit is Adj + Ctl {
        H(qAlice);
        CNOT(qAlice, qBob);
    }

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