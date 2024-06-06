namespace Kata {
    operation PeriodicGivenPeriodOracle (input : Qubit[], target : Qubit, P : Int) : Unit is Adj + Ctl {
        let N = Length(input);
        // Compute XORs of the bits that should be equal in the first N - P qubits
        within {
            for i in 0 .. N - P - 1 {
                CNOT(input[i + P], input[i]);
            }
        } apply {
            // All XORs should be 0s
            ApplyControlledOnInt(0, X, input[... N - P - 1], target);
        }
    }  
    
    operation PeriodicOracle (input : Qubit[], target : Qubit) : Unit is Adj + Ctl {
        let N = Length(input);
        // Check whether the bit string is periodic for any period
        use aux = Qubit[N - 1];
        within {
            for P in 1 .. N - 1 {
                PeriodicGivenPeriodOracle(input, aux[P - 1], P);
            }
        } apply {
            // If any of the aux qubits are 1, the bit string is periodic - use OR
            ApplyControlledOnInt(0, X, aux, target);
            X(target);
        }
    }    
}
