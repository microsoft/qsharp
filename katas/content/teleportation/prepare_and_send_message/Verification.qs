namespace Kata.Verification {
    import Std.Katas.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        return TeleportPreparedStateTestLoop(Kata.PrepareAndSendMessage, ReconstructAndMeasureMessage_Reference);
    }

}