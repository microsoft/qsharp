namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Y, XZ, MinusY, MinusXZ], Kata.DistinguishYfromXZWithPhases, ["Y", "XZ", "-Y", "-XZ"], 1)
    }
}
