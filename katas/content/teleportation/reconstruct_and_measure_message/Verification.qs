namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        return TeleportPreparedStateTestLoop(PrepareAndSendMessage_Reference, Kata.ReconstructAndMeasureMessage);
    }

}