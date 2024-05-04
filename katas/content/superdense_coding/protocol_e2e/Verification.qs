namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        return TestProtocolWithFeedback(Kata.SuperdenseCodingProtocol);
    }
}
