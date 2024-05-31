namespace Kata.Verification {
    @EntryPoint()
    operation CheckSolution() : Bool {
        return MeasurementFreeTeleportTestLoop(Kata.MeasurementFreeTeleport);       
    }
}