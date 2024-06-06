namespace Kata {
    operation PalindromeOracle (input : Qubit[], target : Qubit) : Unit is Adj + Ctl {
        let N = Length(input);
        // Compute XORs of the bits that should be equal in the first qubits
        within {
            for i in 0 .. N / 2 - 1 {
                CNOT(input[N - 1 - i], input[i]);
            }
        } apply {
            // All XORs should be 0s
            ApplyControlledOnInt(0, X, input[... N / 2 - 1], target);
        }
    }    
}
