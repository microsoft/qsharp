namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        return TeleportPreparedStateTestLoop(Kata.PrepareAndSendMessage, ReconstructAndMeasureMessage_Reference);
    }

}