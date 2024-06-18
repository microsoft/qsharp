namespace Kata {
    operation ReconstructMessage_PhiMinus(qBob : Qubit, (b1 : Bool, b2 : Bool)) : Unit {
        if not b1 {
            Z(qBob);
        }
        if b2 {
            X(qBob);
        }
    }
}