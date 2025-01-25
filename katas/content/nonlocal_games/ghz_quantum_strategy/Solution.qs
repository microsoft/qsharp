namespace Kata {
    operation AliceQuantum (bit : Bool, qubit : Qubit) : Bool {
        if bit {
            return MResetX(qubit) == One;
        }
        return MResetZ(qubit) == One;
    }

    operation BobQuantum (bit : Bool, qubit : Qubit) : Bool {
        if bit {
            return MResetX(qubit) == One;
        }
        return MResetZ(qubit) == One;
    }

    // alternative implementation
    operation CharlieQuantum (bit : Bool, qubit : Qubit) : Bool {
        if bit {
            H(qubit);
        }    
        return M(qubit) == One;
    }
}
