namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Y, XZ, -Y, -XZ], Kata.DistinguishYfromXZWithPhases, ["Y", "XZ", "-Y", "-XZ"], 1)
    }
}
