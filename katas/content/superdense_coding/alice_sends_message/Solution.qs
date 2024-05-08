namespace Kata {
    operation EncodeMessageInQubit(qAlice : Qubit, message : (Bool, Bool)) : Unit {
        let (cbit1, cbit2) = message;

        if cbit2 {
            X(qAlice);
        }

        if cbit1 {
            Z(qAlice);
        }
    }
}
