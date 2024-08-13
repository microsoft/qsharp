namespace Kata {
    operation AliceQuantum (bit : Bool, qubit : Qubit) : Bool {
        if bit {
            let res = MResetX(qubit);
            return res == One;
        }
        let res = MResetZ(qubit);
        return res == One;
    }

    operation BobQuantum (bit : Bool, qubit : Qubit) : Bool {
        if bit {
            let res = MResetX(qubit);
            return res == One;
        }
        let res = MResetZ(qubit);
        return res == One;
    }

    // alternative implementation
    operation CharlieQuantum (bit : Bool, qubit : Qubit) : Bool {
        if bit {
            H(qubit);
        }    
        return M(qubit) == One;
    }
}
