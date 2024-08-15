namespace Kata {
    operation OneBitPhaseEstimation(U : Qubit => Unit is Ctl, P : Qubit => Unit) : Int {
        use (control, eigenstate) = (Qubit(), Qubit());
        H(control);
        P(eigenstate);
        Controlled U([control], eigenstate);
        Reset(eigenstate);

        return MResetX(control) == Zero ? 1 | -1;
    }
}
