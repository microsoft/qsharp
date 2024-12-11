namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        let teleport = Kata.StandardTeleport;
        return TeleportTestLoop(teleport);
    }

}
