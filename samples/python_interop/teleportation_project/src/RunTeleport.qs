namespace RunTeleport {

    open TeleportLib;   // references the TeleportLib namespace in Teleport.qs

    @EntryPoint()       // @EntryPoint() not necessary for Python or Jupyter Notebook programs
    operation RunTeleportationExample() : Unit {
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
