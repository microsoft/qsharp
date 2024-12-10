namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Z, MinusZ], Kata.DistinguishZfromMinusZ, ["Z", "-Z"], 1)
    }
}
