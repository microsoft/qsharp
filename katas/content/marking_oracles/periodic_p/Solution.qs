namespace Kata {
    operation Oracle_PeriodicGivenPeriod (x : Qubit[], y : Qubit, p : Int) : Unit is Adj + Ctl {
        let n = Length(x);
        // Compute XORs of the bits that should be equal in the first N - P qubits
        within {
            for i in 0 .. n - p - 1 {
                CNOT(x[i + p], x[i]);
            }
        } apply {
            // All XORs should be 0s
            ApplyControlledOnInt(0, X, x[... n - p - 1], y);
        }
    }    
}
