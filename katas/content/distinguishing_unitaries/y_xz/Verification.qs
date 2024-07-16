namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Y, XZ], Kata.DistinguishYfromXZ, ["Y", "XZ"], 1)
    }
}
