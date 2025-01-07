namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([I, Z], Kata.DistinguishIfromZ, ["I", "Z"], 1)
    }
}
