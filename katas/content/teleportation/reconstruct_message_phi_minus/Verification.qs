namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        let teleport = ComposeTeleportation(StatePrep_BellState(_, _, 1), SendMessage_Reference, Kata.ReconstructMessage_PhiMinus, _, _, _);
        return TeleportTestLoop(teleport);  
    }
}