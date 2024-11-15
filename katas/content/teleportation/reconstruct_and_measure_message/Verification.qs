namespace Kata.Verification {
    import Std.Katas.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        return TeleportPreparedStateTestLoop(PrepareAndSendMessage_Reference, Kata.ReconstructAndMeasureMessage);
    }

}
