namespace Kata.Verification {
    import Std.Diagnostics.*;
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        let teleport = ComposeTeleportation(StatePrep_BellState(_, _, 0), Kata.SendMessage, ReconstructMessage_Reference, _, _, _);
        return TeleportTestLoop(teleport);
    }
}
