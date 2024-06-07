namespace Kata {
    operation PalindromeOracle (x : Qubit[], y : Qubit) : Unit is Adj + Ctl {
        let N = Length(x);
        // Compute XORs of the bits that should be equal in the first qubits
        within {
            for i in 0 .. N / 2 - 1 {
                CNOT(x[N - 1 - i], x[i]);
            }
        } apply {
            // All XORs should be 0s
            ApplyControlledOnInt(0, X, x[... N / 2 - 1], y);
        }
    }    
}
