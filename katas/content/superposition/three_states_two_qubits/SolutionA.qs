namespace Kata {
    operation ThreeStates_TwoQubits (qs : Qubit[]) : Unit {
        use anc = Qubit();
        mutable res = Zero;
        repeat {
            ApplyToEach(H, qs);
            Controlled X(qs, anc);
            set res = MResetZ(anc);
        } 
        until res == Zero
        fixup {
            ResetAll(qs);
        }
    }
}
