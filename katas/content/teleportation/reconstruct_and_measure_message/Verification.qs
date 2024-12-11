namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        return TeleportPreparedStateTestLoop(PrepareAndSendMessage_Reference, Kata.ReconstructAndMeasureMessage);
    }

}
