namespace Kata {
    operation DistinguishTwoQubitUnitaries (unitary : (Qubit[] => Unit is Adj + Ctl)) : Int {
        // First run: apply to |11⟩;
        // CNOT₁₂ will give |10⟩, CNOT₂₁ will give |01⟩, I⊗I and SWAP will remain |11⟩
        use qs = Qubit[2];
        ApplyToEach(X, qs);
        unitary(qs);
        let ind1 = MeasureInteger(qs);

        // Second run: to distinguish I⊗I from SWAP, apply to |01⟩: 
        // I⊗I will remain |01⟩, SWAP will become |10⟩
        X(qs[1]);
        unitary(qs);
        let ind2 = MeasureInteger(qs);

        if ind1 == 1 or ind1 == 2 {
            // Respective CNOT
            return ind1;
        } else {
            return ind2 == 1 ? 3 | 0;
        }
    }
}
