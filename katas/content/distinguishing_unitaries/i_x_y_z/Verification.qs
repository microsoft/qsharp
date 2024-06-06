namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([I, X, Y, Z], Kata.DistinguishPaulis, ["I", "X", "Y", "Z"], 1)
    }
}
