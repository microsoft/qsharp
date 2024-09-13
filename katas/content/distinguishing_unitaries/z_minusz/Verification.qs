namespace Kata.Verification {
    import Std.Katas.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Z, MinusZ], Kata.DistinguishZfromMinusZ, ["Z", "-Z"], 1)
    }
}
