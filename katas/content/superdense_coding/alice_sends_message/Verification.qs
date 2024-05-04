namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        return TestProtocolWithFeedback(ComposeProtocol(Kata.EncodeMessageInQubit, DecodeMessageFromQubits_Reference, _));
    }

}
