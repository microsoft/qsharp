namespace Kata.Verification {
    @EntryPoint()
    operation CheckSolution() : Bool {
        return CheckProtocolWithFeedback(Kata.SuperdenseCodingProtocol);
    }
}
