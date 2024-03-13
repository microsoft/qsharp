namespace Kata {
    open Microsoft.Quantum.Measurement;
    
    operation DeutschAlgorithm (oracle : Qubit => Unit) : Bool {
        use x = Qubit();
        H(x);
        oracle(x);
        H(x);
        return MResetZ(x) == Zero;
    }
}
