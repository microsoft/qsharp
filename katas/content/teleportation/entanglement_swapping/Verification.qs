namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        return TeleportEntanglementSwappingTestLoop(Kata.EntanglementSwapping());
    }
}