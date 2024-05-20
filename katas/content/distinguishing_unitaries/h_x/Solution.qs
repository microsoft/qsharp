namespace Kata {
    open Microsoft.Quantum.Measurement;
    operation DistinguishHX (unitary : (Qubit => Unit is Adj + Ctl)) : Int {
        use q = Qubit();
        within {
            unitary(q);
        } apply {
            X(q);
        }
        return MResetZ(q) == Zero ? 0 | 1;
    }
}
