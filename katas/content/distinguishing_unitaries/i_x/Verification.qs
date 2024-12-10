namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([I, X], Kata.DistinguishIfromX, ["I", "X"], 1)
    }
}
