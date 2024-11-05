namespace Kata {
    operation EncodeMessageInQubit(qAlice : Qubit, message : (Bool, Bool)) : Unit {
        let (bit1, bit2) = message;

        if bit2 {
            X(qAlice);
        }

        if bit1 {
            Z(qAlice);
        }
    }
}
