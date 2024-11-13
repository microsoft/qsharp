namespace RunTeleport {
    open TeleportLib;   // references the TeleportLib namespace in Teleport.qs

    operation Main() : Unit {
        use msg = Qubit();
        use target = Qubit();

        H(msg);
        Teleport(msg, target);    // calls the Teleport() operation from Teleport.qs
        H(target);

        if M(target) == Zero {
            Message("Teleported successfully!");

            Reset(msg);
            Reset(target);
        }
    }
}
