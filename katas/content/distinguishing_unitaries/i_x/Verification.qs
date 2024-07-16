namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([I, X], Kata.DistinguishIfromX, ["I", "X"], 1)
    }
}
