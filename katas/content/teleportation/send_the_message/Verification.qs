namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {        
        return CheckTeleportationWithFeedback(ComposeTeleportation(_,
                                        StatePrep_BellState(_,_,0),
                                        Kata.SendMessage,
                                        ReconstructMessage_Reference,
                                        _,_,_)
                                        );
    }

}