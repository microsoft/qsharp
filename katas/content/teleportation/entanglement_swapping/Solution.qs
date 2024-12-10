namespace Kata {
    import Std.Convert.*;

    operation EntanglementSwapping() : ((Qubit, Qubit) => Int, (Qubit, Int) => Unit) {
        return (SendMessageCharlie, ReconstructMessageBob);
    }

    operation SendMessageCharlie(qAlice1 : Qubit, qBob1 : Qubit) : Int {
        let (c1, c2) = SendMessage(qAlice1, qBob1);
        return BoolArrayAsInt([c1, c2]);
    }

    operation ReconstructMessageBob(qBob2 : Qubit, resultCharlie : Int) : Unit {
        let classicalBits = IntAsBoolArray(resultCharlie, 2);
        ReconstructMessage(qBob2, (classicalBits[0], classicalBits[1]));
    }

    operation SendMessage(qAlice : Qubit, qMessage : Qubit) : (Bool, Bool) {
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
