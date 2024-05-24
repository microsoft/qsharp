namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Z, R(Z, PI)], Kata.DistinguishZfromMinusZ, ["Z", "-Z"], 1)
    }
}
