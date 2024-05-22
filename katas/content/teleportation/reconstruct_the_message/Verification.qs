namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {        
        return CheckTeleportationWithFeedback(ComposeTeleportation(_,
                                        StatePrep_BellState(_,_,0),
                                        SendMessage_Reference,
                                        Kata.ReconstructMessage,
                                        _,_,_)
                                        );
    }
}