namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        let teleport = ComposeTeleportation(StatePrep_BellState(_, _, 3), SendMessage_Reference, Kata.ReconstructMessage_PsiMinus, _, _, _);
        return TeleportTestLoop(teleport);  
    }
}