namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([I, Z], Kata.DistinguishIZ, ["I", "Z"], 1)
    }
}
