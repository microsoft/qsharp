namespace Kata {
    operation AliceQuantum (bit : Bool, qubit : Qubit) : Bool {
        if bit {
            let q = MResetX(qubit);
            return q == One;
        }
        else {
            let q = MResetZ(qubit);
            return q == One;
        }
    }
}
