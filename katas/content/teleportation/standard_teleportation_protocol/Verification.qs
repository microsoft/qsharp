namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        let teleport = Kata.StandardTeleport(_, _, _);
        return TeleportTestLoop(teleport);
    }

}