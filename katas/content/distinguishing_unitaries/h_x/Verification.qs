namespace Kata.Verification {
    import Std.Katas.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([H, X], Kata.DistinguishHfromX, ["H", "X"], 1)
    }
}
