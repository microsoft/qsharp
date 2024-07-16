namespace Kata.Verification {
    @EntryPoint()
    operation CheckSolution() : Bool {
        return CheckProtocolWithFeedback(ComposeProtocol(Kata.EncodeMessageInQubit, DecodeMessageFromQubits_Reference, _));
    }

}
