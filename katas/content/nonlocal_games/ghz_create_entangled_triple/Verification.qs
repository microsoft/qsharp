namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;

    operation CreateEntangledTriple_Reference (qs : Qubit[]) : Unit is Adj {
        X(qs[0]);
        X(qs[1]);

        H(qs[0]);
        H(qs[1]);
        // At this point we have (|000⟩ - |010⟩ - |100⟩ + |110⟩) / 2

        // Flip the sign of the last term
        Controlled Z([qs[0]], qs[1]);

        // Flip the state of the last qubit for the two middle terms
        ApplyControlledOnBitString([false, true], X, [qs[0], qs[1]], qs[2]);
        ApplyControlledOnBitString([true, false], X, [qs[0], qs[1]], qs[2]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        use qs = Qubit[3];
        // apply operation that needs to be tested
        Kata.CreateEntangledTriple(qs);
            
        // apply adjoint reference operation and check that the result is |0ᴺ⟩
        Adjoint CreateEntangledTriple_Reference(qs);
            
        // check that all qubits end up in |0⟩ state
        let result = CheckAllZero(qs);
        ResetAll(qs);
	if result {
            Message("Correct!");
	} 
        else {
            Message("Entangled triple is not implemented correctly");
        }
        return result;
    }
}
