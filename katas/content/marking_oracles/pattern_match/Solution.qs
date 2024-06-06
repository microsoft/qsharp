namespace Kata {
    open Microsoft.Quantum.Arrays;
    
    operation PatternMatchingOracle (input : Qubit[], target : Qubit, indices : Int[], pattern : Bool[]) : Unit is Adj + Ctl {
        // Get the list of qubits that should be used as controls
        let ctrl = Subarray(indices, input);
        ApplyControlledOnBitString(pattern, X, ctrl, target);
    }
}
