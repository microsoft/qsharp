namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        return CheckProtocolWithFeedback(ComposeProtocol(Kata.EncodeMessageInQubit, DecodeMessageFromQubits_Reference, _));
    }

}
