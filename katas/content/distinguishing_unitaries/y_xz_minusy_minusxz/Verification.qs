namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Y, MinusXZ, MinusY, XZ], Kata.DistinguishYfromXZWithPhases, ["Y", "-XZ", "-Y", "XZ"], 1)
    }
}
