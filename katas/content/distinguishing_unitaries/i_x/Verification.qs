namespace Kata.Verification {
    import Std.Katas.*;

    @EntryPoint()
    operation CheckSolution() : Bool {
        DistinguishUnitaries_Framework([I, X], Kata.DistinguishIfromX, ["I", "X"], 1)
    }
}
