namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        return CheckProtocolWithFeedback(ComposeProtocol(EncodeMessageInQubit_Reference, Kata.DecodeMessageFromQubits, _));
    }
}

