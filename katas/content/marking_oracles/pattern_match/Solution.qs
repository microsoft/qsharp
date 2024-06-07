namespace Kata {
    open Microsoft.Quantum.Arrays;

    operation Oracle_PatternMatching (x : Qubit[], y : Qubit, a : Int[], r : Bool[]) : Unit is Adj + Ctl {
        // Get the list of qubits that should be used as controls
        let ctrl = Subarray(a, x);
        ApplyControlledOnBitString(r, X, ctrl, y);
    }
}
