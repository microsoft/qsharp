namespace Kata.Verification {
    @EntryPoint()
    operation CheckSolution() : Bool {
        return CheckProtocolWithFeedback(ComposeProtocol(EncodeMessageInQubit_Reference, Kata.DecodeMessageFromQubits, _));
    }
}

