namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        return TeleportPreparedStateTestLoop(Kata.PrepareAndSendMessage, ReconstructAndMeasureMessage_Reference);
    }

}
