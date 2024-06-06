namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {        
        let teleport = ComposeTeleportation(StatePrep_BellState(_, _, 0), Kata.SendMessage, ReconstructMessage_Reference, _, _, _);
        return TeleportTestLoop(teleport);        
    }
}