namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Y, XZ], Kata.DistinguishHX, ["Y", "XZ"], 1)
    }
}
