namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        return TestProtocolWithFeedback(ComposeProtocol(EncodeMessageInQubit_Reference, Kata.DecodeMessageFromQubits, _));
    }
}

