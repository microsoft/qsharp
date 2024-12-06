namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([Z, S], Kata.DistinguishZfromS, ["Z", "S"], 1)
    }
}
