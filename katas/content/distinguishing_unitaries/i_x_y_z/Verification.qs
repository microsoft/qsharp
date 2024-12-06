namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([I, X, Y, Z], Kata.DistinguishPaulis, ["I", "X", "Y", "Z"], 1)
    }
}
