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
}
