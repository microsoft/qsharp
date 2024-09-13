namespace Kata.Verification {
    import Std.Katas.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        let teleport = Kata.StandardTeleport;
        return TeleportTestLoop(teleport);
    }

}