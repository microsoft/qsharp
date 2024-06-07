namespace Kata {
    operation PeriodicGivenPeriodOracle (x : Qubit[], y : Qubit, p : Int) : Unit is Adj + Ctl {
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
    
    operation PeriodicOracle (x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        let N = Length(x);
        // Check whether the bit string is periodic for any period
        use aux = Qubit[N - 1];
        within {
            for P in 1 .. N - 1 {
                PeriodicGivenPeriodOracle(x, aux[P - 1], P);
            }
        } apply {
            // If any of the aux qubits are 1, the bit string is periodic - use OR
            ApplyControlledOnInt(0, X, aux, y);
            X(y);
        }
    }    
}
