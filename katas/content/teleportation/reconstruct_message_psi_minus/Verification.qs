namespace Kata.Verification {
    import Std.Diagnostics.*;
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        let teleport = ComposeTeleportation(StatePrep_BellState(_, _, 3), SendMessage_Reference, Kata.ReconstructMessage_PsiMinus, _, _, _);
        return TeleportTestLoop(teleport);
    }
}
