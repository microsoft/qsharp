namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Z, MinusZ], Kata.DistinguishZfromMinusZ, ["Z", "-Z"], 1)
    }
}
