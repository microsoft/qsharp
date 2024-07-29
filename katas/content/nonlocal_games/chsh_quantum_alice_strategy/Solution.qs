namespace Kata {
    operation AliceQuantum (bit : Bool, qubit : Qubit) : Bool {
        // Measure in sign basis if bit is 1, and
        // measure in computational basis if bit is 0
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
