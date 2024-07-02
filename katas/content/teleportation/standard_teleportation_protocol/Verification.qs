namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        let teleport = Kata.StandardTeleport;
        return TeleportTestLoop(teleport);
    }

}