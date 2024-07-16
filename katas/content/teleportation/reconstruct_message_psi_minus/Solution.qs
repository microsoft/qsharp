namespace Kata {
    operation ReconstructMessage_PsiMinus(qBob : Qubit, (b1 : Bool, b2 : Bool)) : Unit {
        if not b1 {
            Z(qBob);
        }
        if not b2 {
            X(qBob);
        }
    }
}