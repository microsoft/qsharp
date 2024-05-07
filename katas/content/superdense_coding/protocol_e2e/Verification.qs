namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        return CheckProtocolWithFeedback(Kata.SuperdenseCodingProtocol);
    }
}
