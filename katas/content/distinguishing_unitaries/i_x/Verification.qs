namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([I, X], Kata.DistinguishIX, ["I", "X"], 1)
    }
}
