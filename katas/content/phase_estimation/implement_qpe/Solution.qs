namespace Kata {
    operation PhaseEstimation(
        U : Qubit => Unit is Ctl, 
        P : Qubit => Unit,
        n : Int
    ) : Int {
        use (phaseRegister, eigenstate) = (Qubit[n], Qubit());
        P(eigenstate);
        ApplyToEach(H, phaseRegister);
        for k in 0 .. n - 1 {
            for _ in 1 .. 1 <<< k {
                Controlled U([phaseRegister[k]], eigenstate);
            }
        }
        SwapReverseRegister(phaseRegister);
        Adjoint ApplyQFT(phaseRegister);

        Reset(eigenstate);
        return MeasureInteger(phaseRegister);
    }
}
