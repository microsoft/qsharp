import TeleportOperations.PrepareState.PrepareState.*;     // references the PrepareState.qs file

operation Teleport(msg : Qubit, target : Qubit) : Unit {
    use here = Qubit();

    PrepareBellPair(here, target);      // calls the PrepareBellPair() operation from PrepareState.qs
    Adjoint PrepareBellPair(msg, here);

    if M(msg) == One { Z(target); }
    if M(here) == One { X(target); }

    Reset(here);
}
