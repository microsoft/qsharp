namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Z, S], Kata.DistinguishZfromS, ["Z", "S"], 1)
    }
}
