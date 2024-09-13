namespace Kata.Verification {
    import Std.Katas.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Z, S], Kata.DistinguishZfromS, ["Z", "S"], 1)
    }
}
