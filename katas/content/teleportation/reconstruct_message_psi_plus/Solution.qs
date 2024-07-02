namespace Kata {
    operation ReconstructMessage_PsiPlus(qBob : Qubit, (b1 : Bool, b2 : Bool)) : Unit {
        if b1 {
            Z(qBob);
        }
        if not b2 {
            X(qBob);
        }
    }
}