namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([H, X], Kata.DistinguishHX, ["H", "X"], 1)
    }
}
