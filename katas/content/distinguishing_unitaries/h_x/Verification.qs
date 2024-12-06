namespace Kata.Verification {
    import KatasUtils.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([H, X], Kata.DistinguishHfromX, ["H", "X"], 1)
    }
}
