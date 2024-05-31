namespace Kata {
    operation EntanglementSwapping() : ((Qubit, Qubit) => Int, (Qubit, Int) => Unit) {
        return (TeleportEntanglement, AdjustTeleportedState);
    }
    
    // You might find these skeleton operations useful.
    operation TeleportEntanglement(qAlice1 : Qubit, qBob1 : Qubit) : Int {
        // Implement your solution here...
        return -1;
    } 

    operation AdjustTeleportedState(qBob2 : Qubit, resultCharlie : Int) : Unit {
        // Implement your solution here...
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