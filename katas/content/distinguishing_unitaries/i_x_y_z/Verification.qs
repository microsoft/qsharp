namespace Kata.Verification {
    import Std.Katas.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([I, X, Y, Z], Kata.DistinguishPaulis, ["I", "X", "Y", "Z"], 1)
    }
}
